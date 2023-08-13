mod evm;

use moka::future::Cache;
use poem::{listener::TcpListener, Route, Server};
use poem_openapi::{
    param::Path,
    payload::Json,
    ApiResponse,
    Object,
    OpenApi,
    OpenApiService,
    Union
};
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


struct Api {
    contract: Contract<Http>,
    cache: Cache<u64, Metadata>,
}

#[OpenApi]
impl Api {
    fn new() -> Self {
        let transport = Http::new(&std::env::var("NODE_RPC_URL").unwrap()).unwrap();
        let web3 = Web3::new(transport);
        Self {
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
        if let Ok(result) = self.contract.query::<evm::types::Metadata, _, _, _>(
            "getMetadata",
            (U256::from(*id),),
            None,
            Options::default(),
            None
        ).await {
            GetTokenResponse::Ok(Json(if let Some(metadata) = self.cache.get(&*id) {
                metadata
            } else {
                let metadata = Metadata {
                    image: String::new(),
                    external_url: format!("https://mandelbrot-nft.onrender.com/node/{}", *id),
                    attributes: vec![Attribute {
                        display_type: "number".into(),
                        trait_type: "Locked FUEL".into(),
                        value: Value::Float(result.locked_fuel),
                    }],
                };
                self.cache.insert(*id, metadata.clone()).await;
                metadata
            }))
        } else {
            GetTokenResponse::NotFound
        }
    }
}


#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let api_service = OpenApiService::new(Api::new(), "Hello World", "1.0")
        .server("https://mandelbrot-service.onrender.com")
        .server("http://127.0.0.1:10000");
    let ui = api_service.swagger_ui();
    let app = Route::new().nest("/", api_service).nest("/docs", ui);
    
    Server::new(TcpListener::bind("0.0.0.0:10000"))
        .run(app)
        .await;
}
