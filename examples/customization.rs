use youchoose;

fn main() {
    let mut menu = youchoose::Menu::new(0..100)
        .preview(multiples) // Sets the preview function
        .preview_pos(youchoose::ScreenSide::Bottom, 0.3) // Sets the position of the preview pane
        .preview_label(" multiples ") // Sets the text at the top of the preview pane
        .multiselect() // Allows multiple items to be selected
        .icon(":(") // Sets the default (not selected) icon for an item
        .selected_icon(":)") // The icon for selected items
        .add_multiselect_key('s' as i32) // Bind the 's' key to multiselect
        .add_up_key('u' as i32) // Bind the 'u' key to up
        .add_down_key('d' as i32) // Bind the 'd' key to down
        .add_select_key('.' as i32) // Bind the '.' key to select
        .title(" A custom title ") // Sets the text at the top of the main pane
        .boxed(); // Draws a box around the menu

    let _choice = menu.show();
}

fn multiples(num: i32) -> String {
    // --- Snip ---
    format!("very custom: {}", num)
}
