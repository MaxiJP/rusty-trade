use std::time::{SystemTime, UNIX_EPOCH};

use tungstenite::{connect, Message};
use serde_json::Value;
use colored::Colorize;
use chrono::{Local};
use rusqlite::{Connection, Result, params};

struct MaxTrade {
    sell: bool,
    username: String,
    amount: f64,
    coin: String,
    total_buy: f64,
    single_coin_price: f64,
    trade_type: String,
    time: u64,
    date_print: String,
}


fn add_trade_to_db(trade: &MaxTrade, conn: &Connection) -> Result<()> {
    let mut sell_text;
    if trade.sell {
        sell_text = String::from("SELL")
    } else {
        sell_text = String::from("BUY")
    }

    conn.execute(
        "INSERT INTO trades (timestamp, trade_type_val, action, username, amount, coin_symbol, total_value, price) values
                            (?1,        ?2,             ?3,     ?4,       ?5,     ?6,          ?7,          ?8)",
        params![
            trade.time,
            trade.trade_type,
            sell_text,
            trade.username,
            trade.amount,
            trade.coin,
            trade.total_buy,
            trade.single_coin_price
        ]
    )?;
    Ok(())
}

fn start_database(conn: &Connection) -> Result<()>{

    conn.execute(
        "CREATE TABLE IF NOT EXISTS trades (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp INTEGER NOT NULL,
            trade_type_val TEXT NOT NULL,
            action TEXT NOT NULL,
            username TEXT NOT NULL,
            amount REAL NOT NULL,
            coin_symbol TEXT NOT NULL,
            total_value REAL NOT NULL,
            price REAL NOT NULL
        )",
        []
    )?;
    Ok(())
}

fn main() -> Result<()> {

    let (mut socket, _response) = connect("ws://ws.rugplay.com/api/").expect("Can't connect");
    let conn = Connection::open("trades.db")?;

    let show_live_trade = false;

    socket.send(Message::Text("{\"type\":\"subscribe\",\"channel\":\"trades:all\"}".into())).unwrap();
    socket.send(Message::Text("{\"type\":\"set_coin\",\"coinSymbol\":\"@global\"}".into())).unwrap();
    start_database(&conn);

    loop {
        let msg = socket.read();
        let mut sell;
        let mut username;
        let mut amount;
        let mut coin;
        let mut total_buy;
        let mut single_coin_price;
        let mut trade_type;

        for message in &msg {
            let msg_string = message.to_string();
            let v: Value = serde_json::from_str(&msg_string).expect("REASON");
            if v["data"]["type"] == "SELL" {
                sell = true;
            } else {
                sell = false;
            }
            username = v["data"]["username"].clone();
            amount = v["data"]["amount"].clone().as_f64();
            coin = v["data"]["coinSymbol"].clone();
            total_buy = v["data"]["totalValue"].clone().as_f64();
            single_coin_price = v["data"]["price"].clone().as_f64();
            trade_type = v["type"].clone();

            let trade_type_print = trade_type.as_str().as_slice()[0];
            let date = Local::now();
            let date_print = date.format("%H:%M:%S");
            let mut epoch;

            match SystemTime::now().duration_since(UNIX_EPOCH) {
                Ok(n) => {
                    epoch = n.as_secs();
                },
                Err(_) => panic!("SystemTime before UNIX EPOCH!"),
            }

            if trade_type_print == "ping" {
                break;
            }
            
            let amount_print = amount.as_slice()[0];
            let coin_print = coin.as_str().as_slice()[0];
            let total_buy_print = total_buy.as_slice()[0];
            let single_coin_price_print = single_coin_price.as_slice()[0];
            let username_print = username.as_str().as_slice()[0];

            let trade = MaxTrade {
                sell: sell,
                username: username_print.to_string(),
                amount: amount_print,
                coin: coin_print.to_string(),
                total_buy: total_buy_print,
                single_coin_price: single_coin_price_print,
                trade_type: trade_type_print.to_string(),
                time: epoch,
                date_print: date_print.to_string(),
            };

            add_trade_to_db(&trade, &conn);
            
            if trade_type_print == "live-trade" {
                if !show_live_trade {
                    break;
                }
            }

            if trade.sell == true {
                println!("{} [{}] {} {} {:.2} {} (${:.2}) @ ${:.8}", trade.date_print, trade.trade_type, "SELL".red(), trade.username, trade.amount, trade.coin, trade.total_buy, trade.single_coin_price);
            } else {
                println!("{} [{}] {} {} {:.2} {} (${:.2}) @ ${:.8}", trade.date_print, trade.trade_type, "BUY".green(), trade.username, trade.amount, trade.coin, trade.total_buy, trade.single_coin_price);
            }

        }
        
    }

}

