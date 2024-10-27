# Embedder
This service converts input into embeddings and saves them into the Qdrant vector database.

## Deployment
Use the following command to run `embedder`:

`ADDRESS="0.0.0.0:8080" OLLAMA_ADDRESS="http://192.168.0.104:11434" QDRANT_ADDRESS="http://192.168.0.111:6334" QDRANT_COLLECTION="embeddings" cargo run --release`

| Name | Description | Example |
| - | - | - |
| `ADDRESS` | The address that the local webserver is listening on | `0.0.0.0:8080` | 
| `OLLAMA_ADDRESS` | The address of the ollama server | `http://192.168.0.104:11434` | 
| `QDRANT_ADDRESS` | The address of the Qdrant server | `http://192.168.0.111:6334` |
| `QDRANT_COLLECTION` | The collection the vectors will be stored in | `embeddings` | 
