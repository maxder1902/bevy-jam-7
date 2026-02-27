//! The victory menu.
use bevy::{prelude::*, window::CursorOptions};
use crate::{
    menus::Menu,
    screens::{Screen, set_cursor_grab},
    theme::widget,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Victory), spawn_victory_menu);
}

fn spawn_victory_menu(mut commands: Commands, mut cursor_options: Single<&mut CursorOptions>) {
    set_cursor_grab(&mut cursor_options, false);
    commands.spawn((
        widget::ui_root("Victory Menu"),
        GlobalZIndex(2),
        DespawnOnExit(Menu::Victory),
        children![
            widget::header("Level Complete!"),
            widget::button("Play Again", play_again),
            widget::button("Quit to title", quit_to_title),
        ],
    ));
}

fn play_again(
    _: On<Pointer<Click>>,
    mut next_screen: ResMut<NextState<Screen>>,
) {
    next_screen.set(Screen::Gameplay);
}

fn quit_to_title(_: On<Pointer<Click>>, mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::Title);
}
