use youchoose;

fn main() {
    let mut menu = youchoose::Menu::new(0..100)
    			.preview(multiples)  // Sets the preview function
    			.preview_pos(youchoose::Bottom)  // Sets the position of the preview pane
    			.preview_label(" multiples ")
   
    			.multiselect()  // Allows multiple items to be selected
    			.icon(":(")  // Sets the default (not selected) icon for an item
    			.selected_icon(":)")
    			.add_multiselect_key('s' as i32)
    			.add_up_key('u' as i32)
    			.add_down_key('d' as i32)
    			.add_select_key('.' as i32)
    			
    ;  // Sets the selected icon for an item
    
    
    let choice = menu.show();
}

fn multiples(num: i32) -> String {
    // --- Snip ---
}
