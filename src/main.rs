mod evm;

use moka::future::Cache;
use poem::{endpoint::StaticFilesEndpoint, listener::TcpListener, Result, Route, Server};
use poem_openapi::{
    param::Path,
    payload::Json,
    types::multipart::Upload,
    ApiResponse,
    Multipart,
    Object,
    OpenApi,
    OpenApiService,
    Union
};
use tokio::io::AsyncWriteExt;
use web3::{
    contract::{Contract, Options},
    types::U256,
    transports::Http,
    Web3
};


#[derive(Clone, Debug, PartialEq, Union)]
enum Value {
    Int(u64),
    String(String),
    Float(f64),
}


#[derive(Clone, Debug, Object)]
struct Attribute {
    display_type: String,
    trait_type: String,
    value: Value,
}


#[derive(Clone, Debug, Object)]
struct Metadata {
    image: String,
    external_url: String,
    attributes: Vec<Attribute>,
}


#[derive(Debug, ApiResponse)]
enum GetTokenResponse {
    #[oai(status = 200)]
    Ok(Json<Metadata>),
    #[oai(status = 404)]
    NotFound,
}


// TEMP
#[derive(Debug, Multipart)]
struct UploadPayload {
    file: Upload,
}


struct Api {
    metadata_host: String,
    dapp_host: String,
    contract: Contract<Http>,
    cache: Cache<u64, Metadata>,
}

#[OpenApi]
impl Api {
    /// TEMP: Upload file
    #[oai(path = "/files", method = "post")]
    async fn upload(&self, upload: UploadPayload) -> Result<()> {
        let filename = format!("./images/{}", upload.file.file_name().unwrap().to_string());
        let data = upload.file.into_vec().await.unwrap();
        let mut file = tokio::fs::File::create(filename.clone()).await.unwrap();
        file.write_all(&data).await.unwrap();
        Ok(())
    }

    fn new(metadata_host: String, dapp_host: String) -> Self {
        let transport = Http::new(&std::env::var("NODE_RPC_URL").unwrap()).unwrap();
        let web3 = Web3::new(transport);
        Self {
            metadata_host,
            dapp_host,
            contract: Contract::from_json(
                web3.eth(),
                std::env::var("ERC1155_CONTRACT_ADDRESS").unwrap().trim_start_matches("0x").parse().unwrap(),
                include_bytes!("../resources/MandelbrotNFT.json"),
            ).unwrap(),
            cache: Cache::new(10_000),
        }
    }

    #[oai(path = "/:id", method = "get")]
    async fn get(&self, id: Path<u64>) -> GetTokenResponse {
        if let Some(metadata) = self.cache.get(&*id) {
            GetTokenResponse::Ok(Json(metadata))
        } else {
            if let Ok(metadata) = self.contract.query::<evm::types::Metadata, _, _, _>(
                "getMetadata",
                (U256::from(*id),),
                None,
                Options::default(),
                None
            ).await {
                let image_path = format!("images/{}.png", *id);
                if let Err(error) = mandelbrot_explorer::capture(&image_path, &mandelbrot_explorer::MandelbrotParams {
                    x_min: metadata.field.x_min as f32,
                    x_max: metadata.field.x_max as f32,
                    y_min: metadata.field.y_min as f32,
                    y_max: metadata.field.y_max as f32,
                    max_iterations: 1360
                }).await {
                    eprintln!("{:?}", error);
                }
                let metadata = Metadata {
                    image: format!("{}/{}", self.metadata_host, image_path),
                    external_url: format!("{}/tokens/{}", self.dapp_host, *id),
                    attributes: vec![
                        Attribute {
                            display_type: "number".into(),
                            trait_type: "Parent NFT Id".into(),
                            value: Value::Int(metadata.parent_id as u64),
                        },
                        Attribute {
                            display_type: "number".into(),
                            trait_type: "Locked FUEL".into(),
                            value: Value::Float(metadata.locked_fuel),
                        },
                        Attribute {
                            display_type: "number".into(),
                            trait_type: "Layer".into(),
                            value: Value::Int(metadata.layer as u64),
                        },
                        Attribute {
                            display_type: "number".into(),
                            trait_type: "Depth".into(),
                            value: Value::Float(((metadata.field.x_max - metadata.field.x_min).min(metadata.field.y_max - metadata.field.y_min) / 4.0).log(0.5)),
                        },
                    ],
                };
                self.cache.insert(*id, metadata.clone()).await;
                GetTokenResponse::Ok(Json(metadata))
            } else {
                GetTokenResponse::NotFound
            }
        }
    }
}


#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    std::fs::create_dir_all("./images");

    let metadata_host = std::env::var("METADATA_HOST").unwrap();
    let dapp_host = std::env::var("DAPP_HOST").unwrap();
    let api_service = OpenApiService::new(Api::new(metadata_host.clone(), dapp_host), "Hello World", "1.0")
        .server(metadata_host);
    let ui = api_service.swagger_ui();
    let app = Route::new()
        .nest("/images", StaticFilesEndpoint::new("./images").show_files_listing())
        .nest("/", api_service)
        .nest("/docs", ui);
    
    Server::new(TcpListener::bind("0.0.0.0:10000"))
        .run(app)
        .await;
}
