use std::thread;
use std::sync::{mpsc, Arc, Barrier};
use std::collections::HashMap;
use rand::Rng;

const TOTAL_CARDS: u32 = 48;

fn deal_cards(total_cards: u32, tx_coord_table: &mpsc::Sender<String>) {
    let mut cards = Vec::<String>::new();
    for card_value in 1..13 {
        cards.push(format!("{} de Copas", card_value));
        cards.push(format!("{} de Oros", card_value));
        cards.push(format!("{} de Bastos", card_value));
        cards.push(format!("{} de Espadas", card_value));
    }

    for _ in 0..total_cards {
        let remaining_cards = cards.len();
        let card = cards.remove(rand::thread_rng().gen_range(0, remaining_cards));
        tx_coord_table.send(card).unwrap();
    }
}

fn tell_round_type(round_number: u32, tx_coord_table: &mpsc::Sender<String>) -> String {
    let round_type_number = rand::thread_rng().gen_range(0, 2);
    let round_type;

    if round_type_number == 0 {
        round_type = String::from("silenciosa");
    } else {
        round_type = String::from("hablada");
    }

    println!("Coordinador: 'Se jugará la ronda {}: esta ronda será {}'", round_number, round_type);
    let round_type_message = round_type.clone();
    tx_coord_table.send(round_type_message).unwrap();

    if round_type_number == 1 {
        println!("Coordinador: 'Digan \"Oxidado\" o \"Paso\"'");
    }

    return round_type;
}

fn listen_to_players(round_type: &String, players_number: u32, rx_table_coord: &mpsc::Receiver<String>) -> HashMap::<String, String> {
    let mut choices = HashMap::<String, String>::new();
    if round_type == "hablada" {
        for _ in 0..players_number {
            let player_choice = rx_table_coord.recv().unwrap();
            let data: Vec<&str> = player_choice.split(':').collect();
            println!("* El coordinador escuchó que el jugador {} dijo {} *", data[0], data[1]);
            choices.insert(String::from(data[0]), String::from(data[1]));
        }
    }
    return choices
}

fn get_players_cards(players_number: u32, rx_table_coord: &mpsc::Receiver<String>) -> (HashMap::<String, u32>, String, String) {
    let mut players_cards = HashMap::<String, u32>::new();
    let mut first_player = String::new();
    let mut last_player = String::new();
    for player in 0..players_number {
        let player_card = rx_table_coord.recv().unwrap();
        let data: Vec<&str> = player_card.split(':').collect();
        println!("* El jugador {} colocó la carta \"{}\" sobre el pilón central *", data[0], data[1]);
        if player == 0 {
            first_player = String::from(data[0]);
        } else if player == players_number - 1 {
            last_player = String::from(data[0]);
        }
        let player_card = data[1];
        let data_card: Vec<&str> = player_card.split(' ').collect();
        let card_value = data_card[0].parse::<u32>().unwrap();
        players_cards.insert(String::from(data[0]), card_value);
    }
    return (players_cards, first_player, last_player);
}

fn get_players_with_higher_card(players_cards: HashMap::<String, u32>) -> Vec::<String> {
    let mut max_card = 0;
    let mut max_players = Vec::<String>::new();
    for (player, card) in players_cards {
        if card > max_card {
            max_card = card;
            max_players = vec![player];
        } else if card == max_card {
            max_players.push(player);
        }
    }
    return max_players;
}

fn tell_players_score(round_type: String, scoreboard: &mut Vec::<i32>, choices: &mut HashMap::<String, String>, first_player: String, last_player: String, max_players: Vec::<String>) {
    println!("Coordinador: 'El jugador que apoyó primero su carta es el jugador {}: Recibe 1 punto'", first_player);
    scoreboard[first_player.parse::<usize>().unwrap() - 1] += 1;
    println!("Coordinador: 'El jugador que apoyó último su carta es el jugador {}: Pierde 1 punto'", last_player);
    scoreboard[last_player.parse::<usize>().unwrap() - 1] -= 1;
    for max_player in max_players {
        println!("Coordinador: 'El jugador con la carta más alta es el jugador {}: Recibe 10 puntos'", max_player);
        scoreboard[max_player.parse::<usize>().unwrap() - 1] += 10;
        if round_type == "hablada" && choices.get(&max_player).unwrap() == "Oxidado" {
            choices.remove(&max_player);
            println!("Coordinador: 'El jugador {} había dicho \"Oxidado\" y tiene la carta más alta: Recibe 5 puntos'", max_player);
            scoreboard[max_player.parse::<usize>().unwrap() - 1] += 5;
        }
    }

    if round_type == "hablada" {
        for (player, choice) in choices {
            if choice == "Oxidado" {
                println!("Coordinador: 'El jugador {} había dicho \"Oxidado\" y no tiene la carta más alta: Pierde 5 puntos'", player);
                scoreboard[player.parse::<usize>().unwrap() - 1] -= 5;
            }
        }
    }

    let mut player_id = 1;
    for score in scoreboard {
        println!("Coordinador: 'La puntuación actual del jugador {} es de {} puntos", player_id, score);
        player_id += 1;
    }
}

fn tell_winners(scoreboard: Vec::<i32>) {
    let mut max_score = scoreboard[0];
    let mut winners = Vec::new();
    for player in 0..scoreboard.len() {
        if scoreboard[player] > max_score {
            max_score = scoreboard[player];
            winners = vec![player + 1];
        } else if scoreboard[player] == max_score {
            winners.push(player + 1);
        }
    }

    if winners.len() == 1 {
        println!("Coordinador: '¡El jugador {} es el ganador con {} puntos! ¡Felicidades!'", winners[0], max_score);
    } else {
        println!("Coordinador: '¡Hay empate!'");
        for winner in winners {
            println!("Coordinador: '¡El jugador {} es uno de los ganadores con {} puntos!'", winner, max_score);
        }
        println!("Coordinador: '¡Felicidades!'")
    }
}

pub fn init(players_number: u32,
            table:          thread::JoinHandle<()>,
            tx_coord_table: mpsc::Sender<String>,
            rx_table_coord: mpsc::Receiver<String>,
            barrier:        Arc<Barrier>) -> thread::JoinHandle<()> {

    let rounds = TOTAL_CARDS / players_number;
    let total_cards = rounds * players_number;

    return thread::spawn(move || {
        println!("Coordinador: 'El juego iniciará con {} jugadores'", players_number);

        println!("Coordinador: 'Se jugarán {} rondas'", rounds);

        deal_cards(total_cards, &tx_coord_table);
        barrier.wait();

        let mut scoreboard = vec![0; players_number as usize];
        for round in 0..rounds {
            let round_type = tell_round_type(round + 1, &tx_coord_table);
            barrier.wait();

            let mut choices = listen_to_players(&round_type, players_number, &rx_table_coord);
            barrier.wait();

            let (players_cards, first_player, last_player) = get_players_cards(players_number, &rx_table_coord);

            let max_players = get_players_with_higher_card(players_cards);

            tell_players_score(round_type, &mut scoreboard, &mut choices, first_player, last_player, max_players);

            barrier.wait();
        }

        tell_winners(scoreboard);

        table.join().unwrap();
    });
}
