#![feature(round_char_boundary)]

// C
use core::ffi::c_char;
use std::ffi::{CStr, CString};

// html
use regex::Regex;
use visdom::Vis;

pub mod highlighting;

#[no_mangle]
pub unsafe extern "C" fn cut_off_head(html_str: *const c_char) -> *const c_char {
    //println!("Starting cut_off_head from libreader-rs");
    let html_file_string = CStr::from_ptr(html_str).to_str().unwrap();
    //println!("Html file: {}", html_file_string);
    // unwrap doesn't work
    //let root = Vis::load(html_file_string).unwrap();

    // This gives a strange error... whatever, going into string madness
    //let head = root.find("head");
    //println!("head: \n{}\n", head.html());

    let mut final_str: String = String::new();
    for (pos, item) in html_file_string.split("head").enumerate() {
        //println!("item: {}", item);
        if pos == 1 {
            final_str.push_str("head><head");
        } else {
            final_str.push_str(item);
        }
    }
    //println!("cleaned head: \n{}\n", final_str);

    return Box::leak(CString::new(final_str).unwrap().into_boxed_c_str()).as_ptr();
}

#[no_mangle]
pub unsafe extern "C" fn add_spaces(html_str: *const c_char) -> *const c_char {
    //println!("Starting add_spaces from libreader-rs");

    let html_file_string = CStr::from_ptr(html_str).to_str().unwrap();
    //println!("Html file: {}", html_file_string);

    // unwrap doesn't work
    let root = Vis::load(html_file_string).unwrap();

    let mut text_lines = root.find("p");
    //println!("p lines: \n{}", text_lines.html());

    let font_size_option = text_lines.children("span").attr("style");

    // If this fails, the file is empty propably so just return it
    if font_size_option.is_none() {
        //println!("Font size unknown");
        return html_str;
    }
    // Be scared
    let mut final_font_size_str: String = String::new();
    let font_size_str_is_dot = font_size_option
        .unwrap()
        .to_string()
        .split("font-size:")
        .last()
        .unwrap()
        .to_string();

    //println!("font_size_str_is_dot: {}", font_size_str_is_dot);

    // Diffrent versions of mutool give font size with dot or without it
    if font_size_str_is_dot.contains(".") {
        final_font_size_str = font_size_str_is_dot.split('.').next().unwrap().to_string();
    } else {
        final_font_size_str = font_size_str_is_dot.replace("pt", "");
    }

    //println!("final_font_size_str: {}", final_font_size_str);

    let font_size_res = str::parse::<usize>(&final_font_size_str);

    // Can't get font size, once more
    if font_size_res.is_err() {
        return html_str;
    }
    let font_size = font_size_res.unwrap();

    //println!("Font size: {}", font_size);
    let text_lines_size = text_lines.length();
    //println!("Items count: {:?}", text_lines_size);
    let mut previous_cord: f32 = 0.0;
    // The problem here is that it iterates over span, it could be a problem in the future
    text_lines.for_each(|index, element| {
        //println!("Element {}: \n {}", index, element.text());
        // If line 2 top position - 1 line top position > font size * 2 then add <br>
        // Get top cords

        // get_attribute is nowhere documented ;)...
        let top_cord_split = &element.get_attribute("style").unwrap().to_string();

        let mut top_cord: f32 = 0.0;
        for (pos, item) in top_cord_split.split(";").enumerate() {
            if item.contains("top:") {
                let mut tmp_str: String = item.to_string();
                tmp_str = tmp_str.replace("pt", "").replace("top:", "");
                //println!("tmp_str: {}", tmp_str);
                if tmp_str.contains(".") {
                    // is f32
                    top_cord = str::parse(&tmp_str).unwrap();
                } else {
                    // is i32
                    let tmp_i32: i32 = str::parse(&tmp_str).unwrap();
                    top_cord = tmp_i32 as f32;
                }
            }
        }
        //println!("Top cord {}", top_cord);

        if index == 0 {
            previous_cord = top_cord;
        } else {
            if top_cord - previous_cord >= str::parse::<f32>(&font_size.to_string()).unwrap() * 2.0
            {
                let mut tmp_str = element.html();
                tmp_str = format!("{}{}", "<br>", tmp_str);
                //println!("Added enter \n{}", tmp_str);
                element.set_html(&tmp_str);
            }
            previous_cord = top_cord;
        }

        index <= text_lines_size
    });

    // logs
    //println!("Output is: ");
    /*
    text_lines.for_each(|index, element| {
        println!("{}", element.html());
        index <= text_lines_size
    });
    */

    //println!("Result is: \n{}\n", root.html());

    // Okay so here is a story, when rust became evil
    // This does work, Box leak says to rust to don't free ( the pointer data ) after scope, which is stupid because its returning it, but without it, c++ above doesn't see anything...
    // Also it's a memory leak if the program above doesn't free it
    return Box::leak(CString::new(root.html()).unwrap().into_boxed_c_str()).as_ptr();

    // Here by root.html() if clone is added it returns �7�u�7�u otherwise nothing...
    //return CStr::from_bytes_with_nul(CString::new(root.html().to_owned()).unwrap().to_bytes_with_nul())
    //    .unwrap()
    //    .as_ptr();

    // this one actually says whats the problem ( only this one, and unclear ), but nothing more
    //return CString::new(root.html()).unwrap().as_ptr();

    // Just pure heresy, no comment but worked until the second function wasn't added, which is strange
    //let mut temp_str = root.html();
    //temp_str.push(b'\0' as char);
    //return CStr::from_bytes_with_nul(temp_str.as_bytes()).unwrap().as_ptr();

    // Another thing is that there are like 3 diffrent patchs for c_char?
    // use std::os::raw::c_char;
    // use core::ffi::c_char; - works
    // also the libc one
}
