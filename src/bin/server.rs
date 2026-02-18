use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

struct Player {
    stream: TcpStream,
    reader: BufReader<TcpStream>,
    symbol: char,
}

impl Player {
    fn new(stream: TcpStream, symbol: char) -> std::io::Result<Self> {
        let reader = BufReader::new(stream.try_clone()?);
        Ok(Player { stream, reader, symbol })
    }

    fn send(&mut self, msg: &str) {
        let _ = self.stream.write_all(msg.as_bytes());
        let _ = self.stream.flush();
    }

    fn recv(&mut self) -> Option<String> {
        let mut line = String::new();
        match self.reader.read_line(&mut line) {
            Ok(0) | Err(_) => None,
            Ok(_) => Some(line.trim().to_string()),
        }
    }
}

fn board_display(board: &[char; 9]) -> String {
    let c = |i: usize| {
        if board[i] == ' ' {
            char::from_digit(i as u32, 10).unwrap()
        } else {
            board[i]
        }
    };
    format!(
        " {} | {} | {}\n---+---+---\n {} | {} | {}\n---+---+---\n {} | {} | {}",
        c(0), c(1), c(2),
        c(3), c(4), c(5),
        c(6), c(7), c(8),
    )
}

fn check_winner(board: &[char; 9]) -> Option<char> {
    const WINS: [[usize; 3]; 8] = [
        [0, 1, 2], [3, 4, 5], [6, 7, 8],
        [0, 3, 6], [1, 4, 7], [2, 5, 8],
        [0, 4, 8], [2, 4, 6],
    ];
    for combo in &WINS {
        let c = board[combo[0]];
        if c != ' ' && c == board[combo[1]] && c == board[combo[2]] {
            return Some(c);
        }
    }
    None
}

fn run_game(players: &mut [Player; 2]) {
    let mut board = [' '; 9];
    let mut current = 0usize; // 0 = X, 1 = O

    let position_hint = concat!(
        "Board positions:\n",
        " 0 | 1 | 2\n",
        "---+---+---\n",
        " 3 | 4 | 5\n",
        "---+---+---\n",
        " 6 | 7 | 8\n\n",
    );

    players[0].send(&format!(
        "Opponent connected! You are X. You go first.\n{position_hint}"
    ));
    players[1].send(&format!(
        "Opponent connected! You are O. X goes first.\n{position_hint}"
    ));

    loop {
        let display = format!("\n{}\n\n", board_display(&board));
        players[0].send(&display);
        players[1].send(&display);

        if let Some(winner) = check_winner(&board) {
            let msg = format!("Player {winner} wins!\n");
            players[0].send(&msg);
            players[1].send(&msg);
            let wi = if players[0].symbol == winner { 0 } else { 1 };
            players[wi].send("You WIN! Congratulations!\n");
            players[1 - wi].send("You lose. Better luck next time!\n");
            break;
        }

        if board.iter().all(|&c| c != ' ') {
            players[0].send("It's a DRAW!\n");
            players[1].send("It's a DRAW!\n");
            break;
        }

        // Get move from current player
        let (first, second) = players.split_at_mut(1);
        let (cur, other) = if current == 0 {
            (&mut first[0], &mut second[0])
        } else {
            (&mut second[0], &mut first[0])
        };

        other.send(&format!("Waiting for {} to move...\n", cur.symbol));

        loop {
            cur.send("YOUR_TURN\n");
            match cur.recv() {
                None => {
                    other.send("Opponent disconnected. Game over.\n");
                    return;
                }
                Some(line) => match line.trim().parse::<usize>() {
                    Ok(pos) if pos < 9 && board[pos] == ' ' => {
                        board[pos] = cur.symbol;
                        break;
                    }
                    Ok(pos) if pos < 9 => {
                        cur.send(&format!("Position {pos} is already taken. Try again:\n"));
                    }
                    _ => {
                        cur.send("Invalid input. Enter a number 0-8:\n");
                    }
                },
            }
        }

        current = 1 - current;
    }
}

fn main() {
    let addr = "0.0.0.0:7878";
    let listener = TcpListener::bind(addr).expect("Failed to bind to port 7878");
    println!("=== Tic Tac Toe Server ===");
    println!("Listening on {addr}");

    loop {
        println!("\nWaiting for 2 players to connect...");

        let (mut s1, a1) = listener.accept().expect("Accept failed");
        println!("Player 1 (X) connected from {a1}");
        let _ = s1.write_all(b"Connected! Waiting for second player...\n");
        let _ = s1.flush();

        let (mut s2, a2) = listener.accept().expect("Accept failed");
        println!("Player 2 (O) connected from {a2}");
        let _ = s2.write_all(b"Second player connected!\n");
        let _ = s2.flush();

        thread::spawn(move || {
            let mut players = match (Player::new(s1, 'X'), Player::new(s2, 'O')) {
                (Ok(p1), Ok(p2)) => [p1, p2],
                _ => return,
            };
            run_game(&mut players);
            println!("A game has ended.");
        });
    }
}
