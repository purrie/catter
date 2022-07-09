extern crate termion;
use std::{
    cmp::min,
    env::{args, Args},
    fs::File,
    io::{stdin, stdout, Read, Write},
    iter::Peekable,
};
use termion::{color, terminal_size};
use termion::{
    cursor::Goto, event::Key, input::TermRead, raw::IntoRawMode, screen::AlternateScreen,
};

struct DisplayMods {
    line_count: bool,
    force_single_output: bool,
    interactive_mode_end_of_file: bool,
}

fn main() {
    // first element is path to the binary so if there are less than 2 arguments, we only display help information
    if args().count() < 2 {
        help();
        return;
    }

    // need peekable iterator to peek values in the loop without consuming them
    // It's needed so the iterator can be passed on to display function when it will encounter first file
    let mut arguments = args().peekable();

    // skipping first element since we don't do any work with the path
    arguments.next();

    // setting up flags, there's probably a better way to do this
    let mut display_mods = DisplayMods {
        line_count: false,
        force_single_output: false,
        interactive_mode_end_of_file: false,
    };

    // going over all provided arguments
    while let Some(arg) = arguments.peek() {
        let str_arg = arg.as_str();

        match str_arg {
            // treating help special since it will display help info instead of actual output
            "--help" => {
                help();
                return;
            }
            // program options
            _ if str_arg.starts_with("-") => {
                // filtering out flags to apply their behaviors
                if str_arg.contains("n") {
                    display_mods.line_count = true;
                }
                if str_arg.contains("e") {
                    display_mods.interactive_mode_end_of_file = true;
                }
                if str_arg.contains("o") {
                    display_mods.force_single_output = true;
                }
                if str_arg.contains("h") {
                    help();
                    return;
                }
                // moving to the next element
                arguments.next();
            }
            // generating output for any other, assuming it's a path to a text file
            _ => {
                display_catter(arguments, display_mods);
                return;
            }
        }
    }
}

fn display_catter(mut iter: Peekable<Args>, mods: DisplayMods) {
    // container for all of the text
    let mut full_text = String::new();

    // collecting text from every provided file while skipping invalid files
    while let Some(arg) = iter.next() {
        let mut file = match File::open(&arg) {
            Err(_) => continue,
            Ok(f) => f,
        };
        match file.read_to_string(&mut full_text) {
            Err(_) => {}
            _ => (),
        }
    }

    // displaying requested line count
    if mods.line_count {
        // printing line count only
        let lc = full_text.lines().count();
        println!("{lc}");
        return;
    }

    // printing files contents
    let mut terminal_height = terminal_size().unwrap().1;
    let text_lines = full_text.lines().count() as u16;

    // display the collected text wholesale when it fits terminal or user forces this option
    if text_lines < terminal_height || mods.force_single_output {
        print!("{full_text}");
        return;
    }

    // entering alternate screen o not flood or ruin the main screen
    // using raw mode to get immediate key events since the program doesn't expect actual text input
    let mut screen = AlternateScreen::from(stdout().into_raw_mode().unwrap());

    // all the text lines
    let lines: Vec<&str> = full_text.lines().collect();
    let mut lines_print = (terminal_height - 2) as usize;

    // this will serve as persistant page index
    let mut offset = 0;

    if mods.interactive_mode_end_of_file {
        offset = lines.len() / lines_print;
        offset -= 1;
        if lines.len() % lines_print > 0 {
            offset += 1;
        }
    }

    'outer: loop {
        // clearing the screen to have a blank canvas to print to
        write!(screen, "{}", termion::clear::All).unwrap();
        screen.flush().unwrap();

        // calculating all the information for printing, this should also correctly recalculate pages and ranges if terminal is resized
        terminal_height = terminal_size().unwrap().1;
        lines_print = (terminal_height - 2) as usize;

        let mut pages = lines.len() / lines_print;
        let start = offset * lines_print;
        let mut end = start + lines_print;

        if lines.len() % lines_print > 0 {
            pages += 1;
        }
        if offset == pages - 1 {
            end = lines.len();
        }

        // having less lines than this isn't useful for this mode
        if terminal_height < 3 {
            write!(screen, "{}Terminal height is too small to display the file, resize the terminal and press any button or press q to quit", Goto(1,1)).unwrap();
            screen.flush().unwrap();
        } else {
            // first is a bit of info for the user to help with orientation around the file
            write!(
                screen,
                "{}{}Page {} out of {}{}",
                Goto(1, 1),
                color::Fg(color::Yellow),
                offset + 1,
                pages,
                color::Fg(color::White)
            )
            .unwrap();
            screen.flush().unwrap();

            // this should print all the text lines in desired range
            for (index, text) in lines[start..end].iter().enumerate() {
                write!(screen, "{}{}", Goto(1, index as u16 + 2), text).unwrap();
            }
            screen.flush().unwrap();

            // end line displaying keybinds to navigate the file with
            write!(
                screen,
                "{}{}Press j for next page, k for previous page or q to quit",
                Goto(1, terminal_height),
                color::Fg(color::Yellow)
            )
            .unwrap();
            screen.flush().unwrap();
        }
        // handing the key presses
        'inner: for e in stdin().keys() {
            let evn = e.unwrap();
            match evn {
                Key::Char('j') | Key::Down => {
                    offset = min(offset + 1, pages - 1);
                    break 'inner;
                }
                Key::Char('k') | Key::Up => {
                    if offset > 0 {
                        offset = offset - 1;
                    }
                    break 'inner;
                }
                Key::Char('q') => break 'outer,
                _ => {}
            }
            screen.flush().unwrap();
        }
    }
}
fn help() {
    let help_text = "
    Usage: catter [OPTION]... [FILE]...
    Previews FILE(s).

        -e              interactive mode will start at the last page
        -h, --help      display usage information
        -n              display line count only
        -o              forces long text to be displayed in standard output

    Notes
        Program accepts more than one file, in that case it will concatenate them and output them in the same order as provided
        It will omit any non-text file from output.
        If the output text is longer than available terminal space, it will enter interactive preview mode.

    Examples
        catter text.txt
            Will output text of text.txt

        catter -n text.txt
            Will output an integer that is a number of lines

        catter
            Will display usage information as if -h flag was passed

        catter -o novel.txt
            catter by default enters interactive preview mode for files that won't fit into terminal screen
            -o flag will prevent that and output the text to standard output

    Author
        Written by Purrie Brightstar

    Copyright
        Copyright © 2022 Purrie Brightstar.
        Licence GPLv2: GNU GPL version 2 <https://www.gnu.org/licenses/old-licenses/gpl-2.0.en.html>
        This is free software: you are free to change and redistribute it.
        There is NO WARRANTY, to the extent permitted by law.

    ";

    println!("{}", help_text);
}
