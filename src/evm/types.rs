use ethabi::token::Token;
use web3::{
    contract::tokens::Tokenizable,
    types::{Address, U256},
};


#[derive(Clone, Debug)]
pub struct Field {
    pub x_min: f64,
    pub y_min: f64,
    pub x_max: f64,
    pub y_max: f64,
}

impl Tokenizable for Field {
    fn from_token(token: Token) -> Result<Self, web3::contract::Error> {
        match token {
            Token::Tuple(tokens) => {
                Ok(Self { 
                    x_min: U256::from_token(tokens[0].clone())?.as_u128() as f64 / 10_f64.powi(18) - 2.1,
                    y_min: U256::from_token(tokens[1].clone())?.as_u128() as f64 / 10_f64.powi(18) - 1.5,
                    x_max: U256::from_token(tokens[2].clone())?.as_u128() as f64 / 10_f64.powi(18) - 2.1,
                    y_max: U256::from_token(tokens[3].clone())?.as_u128() as f64 / 10_f64.powi(18) - 1.5
                })
            }
            _ => Err(web3::contract::Error::Abi(ethabi::Error::InvalidData)),
        }
    }

    fn into_token(self) -> Token {
        Token::Tuple(vec![
            U256::from(((self.x_min + 2.1) * 10_f64.powi(18)) as u128).into_token(),
            U256::from(((self.y_min + 1.5) * 10_f64.powi(18)) as u128).into_token(),
            U256::from(((self.x_max + 2.1) * 10_f64.powi(18)) as u128).into_token(),
            U256::from(((self.y_max + 1.5) * 10_f64.powi(18)) as u128).into_token(),
        ])
    }
}

impl web3::contract::tokens::TokenizableItem for Field {}


#[derive(Clone, Debug)]
pub struct Metadata {
    pub token_id: u128,
    pub owner: Address,
    pub parent_id: u128,
    pub field: Field,
    pub locked_fuel: f64,
    pub minimum_price: f64,
    pub layer: u128,
}

impl Tokenizable for Metadata {
    fn from_token(token: Token) -> Result<Self, web3::contract::Error> {
        match token {
            Token::Tuple(tokens) => {
                Ok(Self { 
                    token_id: U256::from_token(tokens[0].clone())?.as_u128(),
                    owner: Address::from_token(tokens[1].clone())?,
                    parent_id: U256::from_token(tokens[2].clone())?.as_u128(),
                    field: Field::from_token(tokens[3].clone())?,
                    locked_fuel: U256::from_token(tokens[4].clone())?.as_u128() as f64 / 10_f64.powi(18),
                    minimum_price: U256::from_token(tokens[5].clone())?.as_u128() as f64 / 10_f64.powi(18),
                    layer: U256::from_token(tokens[6].clone())?.as_u128(),
                })
            }
            _ => Err(web3::contract::Error::Abi(ethabi::Error::InvalidData)),
        }
    }

    fn into_token(self) -> Token {
        Token::Tuple(vec![
            self.token_id.into_token(),
            self.owner.into_token(),
            self.parent_id.into_token(),
            self.field.into_token(),
            U256::from(((self.locked_fuel) * 10_f64.powi(18)) as u128).into_token(),
            U256::from(((self.minimum_price) * 10_f64.powi(18)) as u128).into_token(),
            self.layer.into_token(),
        ])
    }
}

impl web3::contract::tokens::TokenizableItem for Metadata {}
