// chess library imports

use leben_chess::board::piece::{Piece, PieceType};
use leben_chess::board::Board;
use leben_chess::board::board_pos::BoardPosition;
use leben_chess::board::piece::PlayerColor;
use leben_chess::chess::{ChessError, ChessGame, WinReason};
use leben_chess::moves::{ChessMove, PieceMovement};

// ggez imports

use ggez::winit::event_loop;
use ggez::{context, event};
use ggez::graphics::{self, Color, Image, DrawParam};
use ggez::{Context, GameResult};
use ggez::glam::*;
use leben_chess::util::U3;
//use ggez::g


// const variables
const WIDTH: f32 = 1600.0;
const HEIGHT: f32 = 1600.0;






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


fn boardpos_to_guipos(boardpos: BoardPosition) -> BoardPosition { 
    
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

                    let guipos = boardpos_to_guipos(BoardPosition::try_from((row, col)).unwrap());

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


struct GUIMove {
    piece: Piece,
    color: PlayerColor,
    from: BoardPosition,
    to: BoardPosition,
    promotion: Option<Piece>,
}

impl GUIMove {

    

}



struct GameState {
    game: ChessGame,
    board: ChessBoard,
    gameover: bool,
    selected_square: Option<BoardPosition>

}

impl GameState { // set up starting position
    fn new() -> Self {

        GameState {
            game: ChessGame::new(Board::default_board()),
            board: ChessBoard { 
                square_size: (WIDTH / 8.0),
            },
            selected_square: None,
            gameover: false,
        }

    }
}

// implement eventhandler, which requires update and draw functions
impl event::EventHandler for GameState {

    fn update(&mut self, _ctx: &mut Context) -> GameResult {

        if !self.gameover {

            // continue updating

        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> Result<(), ggez::GameError> {

        // canvas that renders to the frame
        let mut canvas = graphics::Canvas::from_frame(
            ctx,
            graphics::Color::from([1.0, 0.0, 0.0, 0.0]),
        );


        self.board.draw(ctx, &mut canvas, self.game.board())?;

        
        canvas.finish(ctx)?;

        Ok(())

        
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
    let (ctx, event_loop) = cb.build()?;
    let state = GameState::new();
    event::run(ctx, event_loop, state);


}