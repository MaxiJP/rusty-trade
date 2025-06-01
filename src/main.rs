use tungstenite::{connect, Message};
use serde_json::Value;
use colored::Colorize;
use chrono::Local;


fn main() {

    let (mut socket, _response) = connect("ws://ws.rugplay.com/api/").expect("Can't connect");

    let show_live_trade = false;

    socket.send(Message::Text("{\"type\":\"subscribe\",\"channel\":\"trades:all\"}".into())).unwrap();
    socket.send(Message::Text("{\"type\":\"set_coin\",\"coinSymbol\":\"@global\"}".into())).unwrap();
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
            let fuck = message.to_string();
            let v: Value = serde_json::from_str(&fuck).expect("REASON");
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

            if trade_type_print == "ping" {
                break;
            }
            
            let amount_print = amount.as_slice()[0];
            let coin_print = coin.as_str().as_slice()[0];
            let total_buy_print = total_buy.as_slice()[0];
            let single_coin_price_print = single_coin_price.as_slice()[0];
            let username_print = username.as_str().as_slice()[0];
            
            if trade_type_print == "live-trade" {
                if !show_live_trade {
                    break;
                }
            }

            if sell == true {
                println!("{} [{}] {} {} {:.2} {} (${:.2}) @ ${:.8}", date_print, trade_type_print, "SELL".red(), username_print, amount_print, coin_print, total_buy_print, single_coin_price_print);
            } else {
                println!("{} [{}] {} {} {:.2} {} (${:.2}) @ ${:.8}", date_print, trade_type_print, "BUY".green(), username_print, amount_print, coin_print, total_buy_print, single_coin_price_print);
            }

        }
        
        // println!("{:?}", print_type_of(&msg));
    }
    // socket.close(None);
}
