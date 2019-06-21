use std::thread;
use std::sync::{mpsc, Arc, Barrier};
use rand::Rng;

const TOTAL_CARDS: u32 = 48;

fn get_dealt_cards(player_number: u32, rounds: u32, rx_table_player: &mpsc::Receiver<String>, tx_output: &mpsc::Sender<String>) -> Vec<String> {
    let mut cards = Vec::<String>::new();
    for _ in 0..rounds {
        let card_received = rx_table_player.recv().unwrap();
        cards.push(card_received);
        tx_output.send(format!("* El jugador {} recibió una carta, ahora tiene {} cartas *", player_number, cards.len())).unwrap();
    }
    return cards;
}

fn listen_round_type(player_number: u32, round_number: u32, rx_table_player: &mpsc::Receiver<String>, tx_output: &mpsc::Sender<String>) -> String {
    let round_type = rx_table_player.recv().unwrap();
    tx_output.send(format!("* El jugador {} escuchó que la ronda {} será {} *", player_number, round_number, round_type)).unwrap();
    return round_type;
}

fn choose_rusty_or_pass() -> String {
    let player_choice_number = rand::thread_rng().gen_range(0, 2);
    let player_choice;
    if player_choice_number == 0 {
        player_choice = String::from("Paso");
    } else {
        player_choice = String::from("Oxidado");
    }
    return player_choice;
}

fn tell_choice_and_listen(player_number: u32, player_choice: String, players_number: u32, tx_player_table: &mpsc::Sender<String>, rx_table_player: &mpsc::Receiver<String>, tx_output: &mpsc::Sender<String>) {
    tx_output.send(format!("Jugador {}: '{}'", player_number, player_choice)).unwrap();
    tx_player_table.send(format!("{}:{}", player_number, player_choice)).unwrap();
    for _ in 0..players_number {
        let other_player_choice = rx_table_player.recv().unwrap();
        let data: Vec<&str> = other_player_choice.split(':').collect();
        if data[0] == format!("{}", player_number) {
            continue;
        }
        tx_output.send(format!("* El jugador {} escuchó que el jugador {} dijo {} *", player_number, data[0], data[1])).unwrap();
    }
}

fn put_card(player_number: u32, cards: &mut Vec<String>, tx_player_table: &mpsc::Sender<String>) {
    let current_card = cards.pop().unwrap();
    tx_player_table.send(format!("{}:{}", player_number, current_card)).unwrap();
}

pub fn init(player_number:      u32,
            players_number:     u32,
            rx_table_player:    mpsc::Receiver<String>,
            tx_player_table:    mpsc::Sender<String>,
            tx_output:          mpsc::Sender<String>,
            barrier:            Arc<Barrier>) -> thread::JoinHandle<()> {

    let rounds = TOTAL_CARDS / players_number;

    return thread::spawn(move || {
        let mut cards = get_dealt_cards(player_number, rounds, &rx_table_player, &tx_output);
        barrier.wait();

        for round in 0..rounds {
            let round_type = listen_round_type(player_number, round + 1, &rx_table_player, &tx_output);
            let player_choice = choose_rusty_or_pass();
            barrier.wait();
            if round_type == "hablada" {
                tell_choice_and_listen(player_number, player_choice, players_number, &tx_player_table, &rx_table_player, &tx_output);
            }
            barrier.wait();
            put_card(player_number, &mut cards, &tx_player_table);

            barrier.wait();
        }
    });
}
