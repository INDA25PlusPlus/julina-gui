// chess library imports

use ggez::winit::event_loop;
use leben_chess::board::Board;
use leben_chess::board::board_pos::BoardPosition;
use leben_chess::board::piece::PlayerColor;
use leben_chess::chess::{ChessError, ChessGame, WinReason};
use leben_chess::moves::{ChessMove, PieceMovement};

// ggez imports

use ggez::event;
use ggez::graphics::{self, Color};
use ggez::{Context, GameResult};
use ggez::glam::*;
//use ggez::g


// const variables
const WIDTH: f32 = 1600.0;
const HEIGHT: f32 = 1600.0;


struct GameState {
    game: ChessGame,
    gameover: bool,

}

impl GameState { // set up starting position
    fn new() -> Self {

        GameState {
            game: ChessGame::new(Board::default_board()),
            gameover: false,
        }

    }
}

// implement eventhandler, which requires update and draw functions
impl event::EventHandler for GameState {

    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> Result<(), ggez::GameError> {

        // canvas that renders to the frame
        let mut canvas = graphics::Canvas::from_frame(
            ctx,
            graphics::Color::from([1.0, 0.0, 0.0, 0.0]),
        );

        /* (0,0) is upper left corner, (0, 1400) bottom left, etc.
        
         */

        let square_size = WIDTH / 8.0; 

        let white_square = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            graphics::Rect::new(0.0, 0.0, square_size, square_size),
             Color::WHITE,
        )?;

        let black_square = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            graphics::Rect::new(0.0, 0.0, square_size, square_size),
            Color::BLACK,
        )?;


        // drawing the board from upper left to bottom right.

        for row in 0..8 {

            for col in 0..8 {

                // calc position of square
                let x = col as f32 * square_size;
                let y = row as f32 * square_size;

                if (row + col) % 2 == 0 {
                    // white square
                    canvas.draw(&white_square, Vec2::new(x, y));
                } else {
                    // black square
                    canvas.draw(&black_square, Vec2::new(x, y));

                }

            }



        }






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
        .window_mode(window_mode);
    let (ctx, event_loop) = cb.build()?;
    let state = GameState::new();
    event::run(ctx, event_loop, state);


}