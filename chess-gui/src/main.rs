// chess library imports

use ggez::winit::event_loop;
use leben_chess::board::Board;
use leben_chess::board::board_pos::BoardPosition;
use leben_chess::board::piece::PlayerColor;
use leben_chess::chess::{ChessError, ChessGame};
use leben_chess::moves::{ChessMove, PieceMovement};

// ggez imports

use ggez::event;
use ggez::graphics::{self, Color};
use ggez::{Context, GameResult};
use ggez::glam::*;
//use ggez::g


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
            graphics::Color::from([0.0, 0.0, 0.0, 1.0]),
        );

        canvas.finish(ctx)?;

        Ok(())

        
    }


}


fn main() -> GameResult {

    let cb = ggez::ContextBuilder::new("chess", "julina");
    let (ctx, event_loop) = cb.build()?;
    let state = GameState::new();
    event::run(ctx, event_loop, state);



}