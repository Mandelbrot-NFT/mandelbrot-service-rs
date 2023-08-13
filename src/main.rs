mod evm;

use poem::{listener::TcpListener, Route, Server};
use poem_openapi::{
    param::Path,
    payload::{Json, PlainText},
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


#[derive(Union, Debug, PartialEq)]
enum Value {
    Int(u64),
    String(String),
    Float(f64),
}


#[derive(Debug, Object)]
struct Attribute {
    display_type: String,
    trait_type: String,
    value: Value,
}


#[derive(Debug, Object)]
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
            ).unwrap()
        }
    }

    #[oai(path = "/:id", method = "get")]
    async fn get(&self, id: Path<u64>) -> GetTokenResponse {
        if let Ok(result) = self.contract.query(
            "getMetadata",
            (U256::from(*id),),
            None,
            Options::default(),
            None
        ).await {
            let metadata: evm::types::Metadata = result;
            GetTokenResponse::Ok(Json(Metadata {
                image: String::new(),
                external_url: format!("https://mandelbrot-nft.onrender.com/node/{}", *id),
                attributes: vec![Attribute {
                    display_type: "number".into(),
                    trait_type: "Locked FUEL".into(),
                    value: Value::Float(metadata.locked_fuel),
                }],
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
