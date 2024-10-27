use crate::ollama::Ollama;
use log::{error, info};
use qdrant_client::{
    qdrant::{
        CreateCollectionBuilder, Distance, PointStruct, UpsertPointsBuilder, Value,
        VectorParamsBuilder, Vectors,
    },
    Qdrant,
};
use serde::Deserialize;
use std::collections::HashMap;
use tokio::sync::{
    mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
    Mutex,
};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize)]
pub struct Input {
    uuid: Uuid,
    text: String,
}

pub struct Embedder {
    ollama: Ollama,
    qdrant: Qdrant,
    sender: UnboundedSender<Input>,
    receiver: Mutex<Option<UnboundedReceiver<Input>>>,
    collection: String,
}

impl Embedder {
    pub async fn new(
        ollama_address: String,
        qdrant_address: String,
        qdrant_collection: String,
    ) -> anyhow::Result<Self> {
        let (sender, receiver) = unbounded_channel();

        let qdrant = Qdrant::from_url(&qdrant_address).build()?;

        if !qdrant.collection_exists(&qdrant_collection).await? {
            qdrant
                .create_collection(
                    CreateCollectionBuilder::new(qdrant_collection.clone())
                        .vectors_config(VectorParamsBuilder::new(1024, Distance::Cosine)),
                )
                .await?;
        }

        Ok(Self {
            ollama: Ollama::from_url(ollama_address)?,
            qdrant,
            sender,
            receiver: Mutex::new(Some(receiver)),
            collection: qdrant_collection,
        })
    }

    pub async fn queue(&self, input: Input) -> anyhow::Result<()> {
        Ok(self.sender.send(input)?)
    }

    pub async fn start(&'static self) {
        let mut lock = self.receiver.lock().await;
        let mut receiver = lock.take().unwrap();
        tokio::task::spawn(async move {
            info!("Successfully started Embedder");

            while let Some(input) = receiver.recv().await {
                let embeddings = match self.ollama.embeddings(&input.text).await {
                    Ok(e) => e,
                    Err(e) => {
                        error!("Couldn't generate embeddings: {}", e);
                        continue;
                    }
                };

                let mut points = Vec::new();

                for (chunk, embedding) in embeddings {
                    points.push(PointStruct {
                        id: None,
                        payload: HashMap::from([
                            ("uuid".to_string(), Value::from(input.uuid.to_string())),
                            ("text".to_string(), Value::from(chunk)),
                        ]),
                        vectors: Some(Vectors::from(embedding)),
                    });
                }

                if let Err(e) = self
                    .qdrant
                    .upsert_points(UpsertPointsBuilder::new(
                        self.collection.to_string(),
                        points,
                    ))
                    .await
                {
                    error!("Couldn't insert vector into qdrant: {}", e);
                }
            }
        });
    }
}
