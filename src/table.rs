use std::thread;
use std::sync::{mpsc, Arc, Barrier};

const TOTAL_CARDS: u32 = 48;

fn get_and_send_dealt_cards(players_number: u32, total_cards: u32, rx_coord_table: &mpsc::Receiver<String>, tx_table_player: &Vec<mpsc::Sender<String>>) {
    let mut current_player = 0;
    for _ in 0..total_cards {
        let card_received = rx_coord_table.recv().unwrap();
        tx_table_player[current_player].send(card_received).unwrap();
        current_player = (current_player + 1) % (players_number as usize);
    }
}

fn get_and_send_round_type(players_number: u32, rx_coord_table: &mpsc::Receiver<String>, tx_table_player: &Vec<mpsc::Sender<String>>) -> String {
    let round_type = rx_coord_table.recv().unwrap();
    for player in 0..players_number {
        let player_round_type = round_type.clone();
        tx_table_player[player as usize].send(player_round_type).unwrap();
    }
    return round_type;
}

fn get_and_send_players_choices(players_number: u32, rx_players_table: &mpsc::Receiver<String>, tx_table_coord: &mpsc::Sender<String>, tx_table_player: &Vec<mpsc::Sender<String>>) {
    for _ in 0..players_number {
        let player_choice = rx_players_table.recv().unwrap();
        let player_choice_message = player_choice.clone();
        tx_table_coord.send(player_choice_message).unwrap();
        for player in 0..players_number {
            let player_choice_message = player_choice.clone();
            tx_table_player[player as usize].send(player_choice_message).unwrap();
        }
    }
}

fn get_and_send_put_cards(players_number: u32, rx_players_table: &mpsc::Receiver<String>, tx_table_coord: &mpsc::Sender<String>) {
    for _ in 0..players_number {
        let player_card = rx_players_table.recv().unwrap();
        tx_table_coord.send(player_card).unwrap();
    }
}

pub fn init(players:            Vec<thread::JoinHandle<()>>,
            rx_coord_table:     mpsc::Receiver<String>,
            tx_table_player:    Vec<mpsc::Sender<String>>,
            rx_players_table:   mpsc::Receiver<String>,
            tx_table_coord:     mpsc::Sender<String>,
            barrier:            Arc<Barrier>) -> thread::JoinHandle<()> {

    let players_number = players.len() as u32;
    let rounds = TOTAL_CARDS / players_number;
    let total_cards = rounds * players_number;

    return thread::spawn(move || {
        get_and_send_dealt_cards(players_number, total_cards, &rx_coord_table, &tx_table_player);
        barrier.wait();

        for _ in 0..rounds {
            let round_type = get_and_send_round_type(players_number, &rx_coord_table, &tx_table_player);

            barrier.wait();
            if round_type == "hablada" {
                get_and_send_players_choices(players_number, &rx_players_table, &tx_table_coord, &tx_table_player);
            }
            barrier.wait();

            get_and_send_put_cards(players_number, &rx_players_table, &tx_table_coord);
            barrier.wait();

        }

        for player in players {
            player.join().unwrap();
        }
    });
}
