// chess library imports

use ggez::event::MouseButton;
//use ggez::winit::dpi::Position;
use leben_chess::board::piece::{Piece, PieceType};
use leben_chess::board::Board;
use leben_chess::board::board_pos::BoardPosition;
use leben_chess::board::piece::PlayerColor;
use leben_chess::chess::{ChessError, ChessGame, WinReason, GameStatus};
use leben_chess::moves::{ChessMove, PieceMovement, PromotionType};

// ggez imports

//use ggez::winit::event_loop;
use ggez::{event};
use ggez::graphics::{self, Canvas, Color, DrawParam, Image};
use ggez::{Context, GameResult};
use ggez::glam::*;
use leben_chess::util::U3;

// tcp imports

use std::io;
use std::io::{Read, Write};
use std::net::{TcpStream, TcpListener};


// constants
const WIDTH: f32 = 1600.0;
const HEIGHT: f32 = 1600.0;
const SQUARE_SIZE: f32 = WIDTH/8.0;


const ADDR: &str = "127.0.0.1:8080";
const MSG_SIZE: usize = 128;


struct ChessPiece {
    piece: Piece,
    position: BoardPosition,
}


impl ChessPiece {


    fn filename(&self) -> &'static str{

        let piece_type = self.piece.piece_type;
        let piece_color = self.piece.player;

        match (piece_type, piece_color) {
            (PieceType::Pawn, PlayerColor::White) => "/wp.png",
            (PieceType::Knight, PlayerColor::White) => "/wN.png",
            (PieceType::Bishop, PlayerColor::White) => "/wB.png",
            (PieceType::Rook, PlayerColor::White) => "/wR.png",
            (PieceType::Queen, PlayerColor::White) => "/wQ.png",
            (PieceType::King, PlayerColor::White) => "/wK.png",
            (PieceType::Pawn, PlayerColor::Black) => "/bp.png",
            (PieceType::Knight, PlayerColor::Black) => "/bN.png",
            (PieceType::Bishop, PlayerColor::Black) => "/bB.png",
            (PieceType::Rook, PlayerColor::Black) => "/bR.png",
            (PieceType::Queen, PlayerColor::Black) => "/bQ.png",
            (PieceType::King, PlayerColor::Black) => "/bK.png",

        }
    }

    fn draw(&self, ctx: &mut Context, canvas: &mut graphics::Canvas, square_size: f32) -> GameResult {

        // draw image of piece on square self.position

        let image_path = Some(self.filename()).unwrap();
        let piece_image = Image::from_path(ctx, image_path)?;
        

        // calc position of square
        let (col, row): (u8, u8) = self.position.into();

        let x = col as f32 * square_size;
        let y = row as f32 * square_size;

        
        let scale = Vec2::new(square_size / piece_image.width() as f32, square_size / piece_image.height() as f32);

        canvas.draw(&piece_image, DrawParam::default()
                            .dest(Vec2::new(x,y))
                            .scale(scale),
        );

        Ok(())
    }
}


fn inverse_boardpos_guipos(boardpos: BoardPosition) -> BoardPosition { 
    
    // on the chessboard, a1 = (0,0) etc., but on the guiboard (0,0) would match square a8
    // this function provides mapping between LERF-mapping and gui image coordinates
    
    let (col, row): (u8, u8) = boardpos.into();

    return BoardPosition {file: U3::try_from(col).unwrap(), rank: U3::try_from(7-row).unwrap()};

}

struct ChessBoard {

    square_size: f32,

}

impl ChessBoard {

    fn draw(&self, ctx: &mut Context, canvas: &mut graphics::Canvas, board: &Board) -> GameResult {

        // draw squares
        self.draw_squares(ctx, canvas)?;

        // then draw pieces

        for row in 0..8 {

            for col in 0..8 {

                if let Some(temp_piece) = Board::get_piece(board, BoardPosition::try_from((row, col)).unwrap()) {

                    let guipos = inverse_boardpos_guipos(BoardPosition::try_from((row, col)).unwrap());

                    let gui_piece = ChessPiece {
                        piece: temp_piece,
                        position: guipos,
                    };
                    gui_piece.draw(ctx, canvas, self.square_size)?;

                }
            }
        }


        Ok(())

    }




    fn draw_squares(&self, ctx: &mut Context, canvas: &mut graphics::Canvas) -> GameResult {

        let white_square = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            graphics::Rect::new(0.0, 0.0, self.square_size, self.square_size),
             Color::from_rgb(220, 220, 220),
        )?;

        let black_square = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            graphics::Rect::new(0.0, 0.0, self.square_size, self.square_size),
            Color::from_rgb(50, 50, 50),
        )?;

        // (0,0) is upper left corner, (0, 1400) bottom left, etc.
        // drawing the board from upper left to bottom right.

        for row in 0..8 {

            for col in 0..8 {

                // calc position of square
                let x = col as f32 * self.square_size;
                let y = row as f32 * self.square_size;

                if (row + col) % 2 == 0 {
                    // white square
                    canvas.draw(&white_square, Vec2::new(x, y));
                } else {
                    // black square
                    canvas.draw(&black_square, Vec2::new(x, y));

                }
            }
        }

        Ok(())
    }



}


struct GameState {
    game: ChessGame,
    board: ChessBoard,
    gameover: bool,
    selected_square: Option<BoardPosition>,
    selected_target: Option<BoardPosition>,
    highlight: Highlight,
    show_gameover_popup: bool,
    promotion: bool,
    network_player: Option<NetworkPlayer>,

}

impl GameState { // set up starting position
    fn new(ctx: &mut Context, network_game: bool, addr: &str) -> GameResult<Self> {

        let network_player = if network_game{
            Some(NetworkPlayer::auto(addr)?)
        } else {
            None
        };

        Ok(GameState {
            game: ChessGame::new(Board::default_board()),
            board: ChessBoard { 
                square_size: (WIDTH / 8.0),
            },
            gameover: false,
            selected_square: None,
            selected_target: None,
            highlight: Highlight::new(ctx).unwrap(),
            show_gameover_popup: false,
            promotion: false,
            network_player,
        })

    }

    fn reset(&mut self, ctx: &mut Context) -> GameResult {

        self.game = ChessGame::new(Board::default_board());

        self.selected_square = None;
        self.highlight.selected_square = None;

        self.gameover = false;
        self.show_gameover_popup = false;

        Ok(())
    }



}

 


fn calc_square_pos (position: BoardPosition) -> Vec2 { // based on board position (not gui board position)

    let (file, rank): (u8, u8) = position.into();

     // calc position of square
    let x =  file as f32 * SQUARE_SIZE;
    let y =  rank as f32 * SQUARE_SIZE;

    Vec2::new(x, y)
}

struct Highlight {

    selected_square: Option<BoardPosition>,
    mesh: Option<graphics::Mesh>,

}

impl Highlight {

    fn new(ctx: &mut Context) -> GameResult<Self> {

        let mesh = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            graphics::Rect::new(0.0, 0.0, SQUARE_SIZE, SQUARE_SIZE),
            Color::from_rgba(255, 255, 0, 128),
        )?;

        Ok(Highlight {
            selected_square: None,
            mesh: Some(mesh),
        })
    }


    fn draw (&self, canvas: &mut Canvas) -> GameResult{

       
        if let Some(boardpos) = self.selected_square {

            if let Some(mesh) = &self.mesh {

                let gui_position = inverse_boardpos_guipos(boardpos);
                let squares_pos = calc_square_pos(gui_position);
                canvas.draw(mesh, squares_pos);

            }
        }

        Ok(())
    }

}
pub enum Role {
    Server,
    Client,
}
struct NetworkPlayer {
    stream: TcpStream,
    role: Role,
    color: PlayerColor,
}

impl NetworkPlayer {
    pub fn auto(addr: &str) -> io::Result<Self> { // auto determine client and server based on connection success

        // Try client
        match TcpStream::connect(addr) {
            Ok(stream) => {
                println!("Connected to {} as CLIENT (White)", addr);
                stream.set_nonblocking(true)?;
                return Ok(NetworkPlayer {stream, role: Role::Client, color: PlayerColor::White});
            }

            Err(e) => {
                if e.kind() != io::ErrorKind::ConnectionRefused {
                    return Err(e);
                }

                // Start as server
                let listener = TcpListener::bind(addr)?;
                println!("Listening on {} as SERVER (Black)", addr);
                println!("Waiting for client to connect...");
                let (stream, sock_addr) = listener.accept()?;
                println!("Client connected from {}", sock_addr);
                stream.set_nonblocking(true)?;
                return Ok(NetworkPlayer {stream, role: Role::Server, color: PlayerColor::Black});
            }
        }

    }

    // fixed size of buffer: 128 bytes
    // five parts separated by ':'
    // Message identifier: 9 characters, "ChessMOVE"
    // Move: 5 characters, eg. A1A50 (capital letters), the last 
    // character indicating promotion piece type
    // Game state: "0-0" ongoing, "1-0" white won, "0-1" black won, "1-1" draw
    // New board: FEN-notation
    // Padding such that total num of bytes is 128

    fn read_tcp_message(&mut self) -> io::Result<Option<String>> {

        let mut msg_buf = [0; MSG_SIZE];

        match self.stream.read_exact(&mut msg_buf) {
            Ok(_) => {
                let msg = String::from_utf8_lossy(&msg_buf).
                    to_string();
                return Ok(Some(msg))
            },
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => Ok(None),
            Err(e) => panic!("IO error: {}", e),
        }


    }

    fn write_tcp_message(&mut self, msg: &str) {

        match self.stream.write(msg.as_bytes()) {

            Ok(_) => {println!("Move sent to opponent!")},
            Err(e) => {println!("Failed to write message: {}", e)}
        }

    }



}



struct HelperNetworkPlayer {
    network_player: NetworkPlayer, 
    game_state: GameState,
}

impl HelperNetworkPlayer {

    fn decode_move(chess_move: &str) -> Option<ChessMove> {

        let from = BoardPosition::try_from(&chess_move[0..2]).ok()?;
        let to = BoardPosition::try_from(&chess_move[2..4]).ok()?;

        let promotion = match chess_move.chars().nth(4) {

            Some('K') => Some(PromotionType::Knight),
            Some('B') => Some(PromotionType::Bishop),
            Some('R') => Some(PromotionType::Rook),
            Some('Q') => Some(PromotionType::Queen),
            Some('0') | None | _ => None,
        };

        return Some(ChessMove{piece_movement: PieceMovement{from, to}, promotion});

    }

    fn encode_move(mv: ChessMove) -> String {

        let files = ["A", "B", "C", "D", "E", "F", "G", "H"];
        let ranks = ["1", "2", "3", "4", "5", "6", "7", "8"];

        let move_string: String = format!(
            "{}{}{}{}{}",
            files[mv.piece_movement.from.file.get() as usize],
            ranks[mv.piece_movement.from.rank.get() as usize],
            files[mv.piece_movement.to.file.get() as usize],
            ranks[mv.piece_movement.to.rank.get() as usize],
            match mv.promotion {
                Some(PromotionType::Knight) => "K",
                Some(PromotionType::Bishop) => "B",
                Some(PromotionType::Rook) => "R",
                Some(PromotionType::Queen) => "Q",
                None => "0",
            }
        );

        return move_string;
    }

    fn board_to_fen(game: &ChessGame) -> String {

        // FEN-notation, excluding castling, en passant etc. (tracked by chess lib)
        // code inspo from leben-chess impl Display for Board

        let mut fen_board = String::new();

        let mut empty_squares = 0;
        for rank in (0..8).rev() { // print eight rank first

            if rank < 7 {
                if empty_squares != 0 {
                    fen_board += &empty_squares.to_string();
                }
                fen_board += "/";
            }
            empty_squares = 0;
            for file in 0..8 { // print a8, b8, ..., h8 etc.

                let pos = BoardPosition{
                    file: file.try_into().unwrap(),
                    rank: rank.try_into().unwrap(),
                };

                let piece = game.board().get_piece(pos);

                if let Some(piece) = piece {
                    if empty_squares != 0 {
                        fen_board += &empty_squares.to_string();
                        empty_squares = 0;
                    }
                    fen_board += piece.get_char();

                } else {
                    empty_squares += 1;
                }

            }
        }

        println!("{}", fen_board);

        return fen_board;
        
    }


     fn decode_message(msg: &str) -> Result<(&str, &str, &str), &'static str>{

        // if (msg).len() < 128 {
        //     return Err("Message too short");
        // }

        let parts: Vec<&str> = msg.trim().split(":").collect();

        println!("{:?}", parts);

        if parts.len() < 4 {
            return Err("Invalid message format");
        }

        if parts[0].to_string() != "ChessMOVE".to_string() {
            return Err("invalid message ID");
        }

        Ok((parts[1], parts[2], parts[3]))
    }



    fn encode_message(game: &ChessGame, mv: ChessMove) -> String{  

        //let separator = ":";
        //let msg_id = "ChessMOVE";

        let game_status = match game.game_status() {
            
            GameStatus::NotYetStarted | GameStatus::Normal => "0-0",
            GameStatus::Win(PlayerColor::White, _) => "1-0",
            GameStatus::Win(PlayerColor::Black, _) => "0-1",
            GameStatus::Draw(_) => "1-1",
        };

        let encoded_move = HelperNetworkPlayer::encode_move(mv);

        let fen = HelperNetworkPlayer::board_to_fen(game); // Make FEN string

        return format!(
            "ChessMOVE:{}:{}:{}:{}",
            encoded_move,
            game_status,
            fen,
            "0".repeat(MSG_SIZE-9-5-3-fen.len()-4)
        );
    }

}




// implement eventhandler, which requires update and draw functions
impl event::EventHandler for GameState {

    fn update(&mut self, _ctx: &mut Context) -> GameResult {

        match self.game.game_status(){

            GameStatus::Win(_,_) => {self.gameover = true;}
            GameStatus::Draw(_) => {self.gameover = true;}

            _ => {}

        }

        if self.gameover {
            // Game over, give user option to restart the game
            self.show_gameover_popup = true;
        }


        // If we're waiting for the opponent to make a move (networking)

        if let Some(network_player) = &mut self.network_player {

            // only read their message if it's their turn
            if network_player.color == self.game.active_player() {
                return Ok(());
            }

            if let Some(msg) = network_player.read_tcp_message()? {


                match HelperNetworkPlayer::decode_message(&msg) {

                    Ok((chess_move, game_state, new_board)) => {

                        if game_state != "0-0" {
                            self.gameover = true;
                            return Ok(());
                        } 

                        let decoded_move = HelperNetworkPlayer::decode_move(chess_move).unwrap();

                        // perform move
                        match self.game.do_move(decoded_move) {

                            Ok(_) => {},
                            Err(e) => {
                                println!("Failed to perform opponent's move: {}", e);
                                // RAGE QUIT
                                self.gameover = true;
                                self.network_player = None; // drop connection
                                return Ok(());
                            }
                        }

                        // compare your new board with opponent's new board

            
                    }

                    Err(e) => {
                        println!("Error decoding message: {}", e);
                        return Ok(())
                    }
                }

            }
        }
       


        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> Result<(), ggez::GameError> {

         // canvas that renders to the frame
        let mut canvas = graphics::Canvas::from_frame(
            ctx,
            graphics::Color::from([1.0, 0.0, 0.0, 0.0]),
        );

        self.board.draw(ctx, &mut canvas, &self.game.board())?;

        self.highlight.draw(&mut canvas)?;


        if self.show_gameover_popup {

            let overlay = graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                graphics::Rect::new(0.0,0.0, WIDTH/4.0, HEIGHT/4.0),
                Color::from_rgba(120, 0, 0, 160),
            )?;
            canvas.draw(&overlay, Vec2::new(SQUARE_SIZE*3.0, SQUARE_SIZE*3.0));

            let msg = format!("Game over!\n{}\nClick to restart.", self.game.game_status());
            let mut text = graphics::Text::new(msg);


            text.set_scale(32.0);
            let text_x = SQUARE_SIZE*3.0 + SQUARE_SIZE/2.0 - (text.measure(ctx).unwrap().y)/2.0;
            let text_y = SQUARE_SIZE*3.0 + SQUARE_SIZE/2.0;
            canvas.draw(&text, DrawParam::default().dest([text_x, text_y]));

        }

        if self.promotion {

            let overlay = graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                graphics::Rect::new(0.0,0.0, WIDTH/4.0, HEIGHT/4.0),
                Color::from_rgba(0, 100, 0, 160),
            )?;
            canvas.draw(&overlay, Vec2::new(SQUARE_SIZE*3.0, SQUARE_SIZE*3.0));

            
            match self.game.active_player() {

                PlayerColor::White => {

                    let knight = ChessPiece{
                        piece: Piece{piece_type: PieceType::Knight, player: PlayerColor::White},
                        position: BoardPosition {file: U3::try_from(3).unwrap(), rank: U3::try_from(4).unwrap()}
                    };
                    let bishop = ChessPiece { 
                        piece: Piece{piece_type: PieceType::Bishop, player: PlayerColor::White},
                        position: BoardPosition {file: U3::try_from(4).unwrap(), rank: U3::try_from(4).unwrap()}
                    };
                    let rook = ChessPiece { 
                        piece: Piece{piece_type: PieceType::Rook, player: PlayerColor::White},
                        position: BoardPosition {file: U3::try_from(3).unwrap(), rank: U3::try_from(3).unwrap()}
                    };
                    let queen = ChessPiece { 
                        piece: Piece{piece_type: PieceType::Queen, player: PlayerColor::White},
                        position: BoardPosition {file: U3::try_from(4).unwrap(), rank: U3::try_from(3).unwrap()}
                    };

                    let promotion_pieces = [knight, bishop, rook, queen];

                    for piece in promotion_pieces {

                        let image_path = Some(piece.filename()).unwrap();
                        let piece_image = Image::from_path(ctx, image_path)?;

                        let scale = Vec2::new(SQUARE_SIZE / piece_image.width() as f32, SQUARE_SIZE / piece_image.height() as f32);

                        let (col, row): (u8, u8) = piece.position.into();
                        let x = col as f32 * SQUARE_SIZE;
                        let y = row as f32 * SQUARE_SIZE;
        
                        canvas.draw(&piece_image, DrawParam::default()
                                    .dest(Vec2::new(x as f32, y as f32))
                                    .scale(scale));
                    }
                }
                
                PlayerColor::Black => {

                    let knight = ChessPiece{
                        piece: Piece{piece_type: PieceType::Knight, player: PlayerColor::Black},
                        position: BoardPosition {file: U3::try_from(3).unwrap(), rank: U3::try_from(5).unwrap()}
                    };
                    let bishop = ChessPiece { 
                        piece: Piece{piece_type: PieceType::Bishop, player: PlayerColor::Black},
                        position: BoardPosition {file: U3::try_from(4).unwrap(), rank: U3::try_from(5).unwrap()}
                    };
                    let rook = ChessPiece { 
                        piece: Piece{piece_type: PieceType::Rook, player: PlayerColor::Black},
                        position: BoardPosition {file: U3::try_from(3).unwrap(), rank: U3::try_from(4).unwrap()}
                    };
                    let queen = ChessPiece { 
                        piece: Piece{piece_type: PieceType::Queen, player: PlayerColor::Black},
                        position: BoardPosition {file: U3::try_from(4).unwrap(), rank: U3::try_from(4).unwrap()}
                    };

                

                    let promotion_pieces = [knight, bishop, rook, queen];

                    for piece in promotion_pieces {

                        let image_path = Some(piece.filename()).unwrap();
                        let piece_image = Image::from_path(ctx, image_path)?;

                        let scale = Vec2::new(SQUARE_SIZE / piece_image.width() as f32, SQUARE_SIZE / piece_image.height() as f32);

                        let (col, row): (u8, u8) = piece.position.into();
                        let x = col as f32 * SQUARE_SIZE;
                        let y = row as f32 * SQUARE_SIZE;

                        canvas.draw(&piece_image, DrawParam::default()
                                    .dest(Vec2::new(x as f32, y as f32))
                                    .scale(scale));
                    }

                }


            }

        }


        canvas.finish(ctx)?;

        Ok(())
        
    }


    fn mouse_button_down_event( // https://docs.rs/ggez/latest/ggez/event/trait.EventHandler.html#method.mouse_button_down_event
            &mut self,
            _ctx: &mut Context,
            _button: event::MouseButton,
            _x: f32, // corresponds to column
            _y: f32, // corresponds to row
        ) -> Result<(), ggez::GameError> {

        match _button {
            MouseButton::Left => {

                if let Some(network_player) = &self.network_player {

                    if network_player.color != self.game.active_player() {
                        println!("Opponent is to move");
                        return Ok(());
                    }
                }


                if !self.show_gameover_popup {

                    // convert (x,y)-coordinates to GuiPosition
                    let row = (_y / SQUARE_SIZE).floor() as u8;
                    let col = (_x / SQUARE_SIZE).floor() as u8;
                    
                    let gui_position = BoardPosition {file: U3::try_from(col).unwrap(), rank: U3::try_from(row).unwrap()};
                    let board_position = inverse_boardpos_guipos(gui_position);

                    let rank = board_position.rank.get(); 
                    let file = board_position.file.get();


                    if self.selected_square == None {

                        // if the square contains a piece with valid moves, select it
                        // game.available_moves(BoardPosition::try_from((row,col)).unwrap()); returns a bitmap containing all zeroes if there's no available moves.

                        let bitboard = self.game.available_moves(BoardPosition::try_from((file, rank)).unwrap()); // (file, rank)
                        
                        if !bitboard.is_all_zeros() {

                            self.selected_square = Some(board_position);
                            self.highlight.selected_square = Some(board_position);
                        }

                    } else if self.selected_target == None { // normal move

                        // now the player clicks the target square, check if the target square is valid

                        let selected = self.selected_square.unwrap();
                        let selected_rank = selected.rank.get();
                        let seleceted_file = selected.file.get();

                        let targeted_rank = rank;
                        let targeted_file = file;

                        self.selected_target = Some(board_position);


                        let from = BoardPosition::try_from((seleceted_file, selected_rank)).unwrap();
                        let to = BoardPosition::try_from((targeted_file, targeted_rank)).unwrap();

                        // check if move is promotion

                        let promotion_expected = self.game.expects_promotion_move(from);

                        if promotion_expected {  // they have to pick a promotion piece type first.
                            self.promotion = true;
                            return Ok(());
                        }

                        let mv = ChessMove {
                            piece_movement: PieceMovement {
                                from: from,
                                to: to,
                            },
                            promotion: None,
                        };

                        match self.game.do_move(mv) {
                            Ok(_) => {
                                println!("Move executed!");
                            }
                            Err(err) => {
                                println!("Illegal move: {:?}", err);
                            }
                        }

                        self.selected_square = None;
                        self.selected_target = None;
                        self.highlight.selected_square = None;


                        if let Some(network_player) = &mut self.network_player {

                            let mv_tcp = HelperNetworkPlayer::encode_message(&self.game, mv);
                            NetworkPlayer::write_tcp_message(network_player, &mv_tcp);
                        }
                        
                        

                    } else if self.promotion {

                        let promotion_type = match (rank, row) {

                            (3, 3) => PromotionType::Knight,
                            (3, 4) => PromotionType::Bishop,
                            (4, 3) => PromotionType::Rook,
                            (4, 4) => PromotionType::Queen,

                            _ => {return Ok(())},
                        };

                        let selected_square = self.selected_square.unwrap();
                        let selected_rank = selected_square.rank.get();
                        let seleceted_file = selected_square.file.get();

                        let selected_target = self.selected_target.unwrap();
                        let targeted_rank = selected_target.rank.get();
                        let targeted_file = selected_target.file.get();

                        let from = BoardPosition::try_from((seleceted_file, selected_rank)).unwrap();
                        let to = BoardPosition::try_from((targeted_file, targeted_rank)).unwrap();

                        let mv = ChessMove {
                            piece_movement: PieceMovement {
                                from: from,
                                to: to,
                            },
                            promotion: Some(promotion_type),
                        };

                        match self.game.do_move(mv) {
                            Ok(_) => {
                                println!("Move executed!");
                            }
                            Err(err) => {
                                println!("Illegal move: {:?}", err);
                            }
                        }

                        self.selected_square = None;
                        self.selected_target = None;
                        self.highlight.selected_square = None;
                        self.promotion = false;

                        if let Some(network_player) = &mut self.network_player {

                            let mv_tcp = HelperNetworkPlayer::encode_message(&self.game, mv);
                            NetworkPlayer::write_tcp_message(network_player, &mv_tcp);
                        }

                    }

                } else {

                    // check if they click "restart game"-button
                    // convert (x,y)-coordinates to GuiPosition
                    let row = (_y / SQUARE_SIZE).floor() as u8;
                    let col = (_x / SQUARE_SIZE).floor() as u8;
                    
                    let gui_position = BoardPosition {file: U3::try_from(col).unwrap(), rank: U3::try_from(row).unwrap()};
                    let board_position = inverse_boardpos_guipos(gui_position);

                    let rank = board_position.rank.get(); 
                    let file = board_position.file.get();

                    match (rank, file) {

                        (3, 3) | (3, 4) | (4, 3) | (4, 4) => {

                            self.reset(_ctx)?;
                            return Ok(());
                        }
                        _ => {}
                    }
                }

                Ok(())
            }

            _ => {
                // Other button is clicked, do nothing
                Ok(())
            } 
        }
        
    }


}


fn main() -> GameResult {

    let window_setup = ggez::conf::WindowSetup::default().title("Chess");
    let window_mode = ggez::conf::WindowMode::default()
        .dimensions(WIDTH, HEIGHT); // width & height of frame

    let cb = ggez::ContextBuilder::new("chess", "julina")
        .window_setup(window_setup)
        .window_mode(window_mode)
        .add_resource_path("./resources");
    let (mut ctx, event_loop) = cb.build()?;
    let state = GameState::new(&mut ctx, true, ADDR)?;
    event::run(ctx, event_loop, state);


}