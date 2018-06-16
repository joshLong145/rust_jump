
extern crate termion;
extern crate rand;

use termion::raw::IntoRawMode;
use std::io::{Read, Write, stdout, Error};
use termion::async_stdin;
use std::time::{Duration,SystemTime};
use std::thread::sleep;
use std::fs;
use rand::Rng;

struct Obj{
    x: u16,
    y: u16,
    sprite: char,
}

struct Player{
    x: u16,
    y: u16,
    sprite: char,
    jump: bool,
    score: i64,
}

struct Time {
    total_time: SystemTime,
}

fn drawplayer(_player: &Player){
    // Get the standard output stream and go to raw mode.
    let mut stdout = stdout().into_raw_mode().unwrap();

    write!(stdout, "{}{}{}{}",
           termion::clear::All,
           termion::cursor::Goto(_player.x + 1, _player.y),
           termion::cursor::Hide,_player.sprite).unwrap();
}

fn playerjump(_player: &mut Player, _terminal_size: &mut Result<(u16,u16), Error>){
    _player.y -= 1;
}

fn update(_player: &mut Player, _enemies: &mut Vec<Obj>, _stars: &mut Vec<(u16,u16)>, _time: &mut Time, _terminal_size: &mut Result<(u16,u16), Error>) -> bool{
  let mut stdout = stdout().into_raw_mode().unwrap();
  let updated_time = SystemTime::now();
  match _terminal_size{
        Ok(s) =>{
        if _player.y < s.1 -1 && !_player.jump{
            _player.y += 1;
        }
        for i in 0.._enemies.len(){
            _enemies[i].x -= 1;
            if(_enemies[i].x == _player.x) && (_enemies[i].y == _player.y){
                write!(stdout, "{}{}{}",termion::cursor::Goto(1,1), termion::clear::All, termion::cursor::Show).unwrap();
                return false;
            }else {
                _player.score += 1;
            }
        }

        for i in 0.._stars.len(){
            _stars[i].0 -= 1u16;

            if _stars[i].0 <= 1 {
                _stars[i].0 = s.0;
            }
        }

        _enemies.retain(move |x|x.x >= 1);

        match updated_time.duration_since(_time.total_time){
            Ok(t) =>{
                if t.as_secs() > 2{
                    _enemies.push(Obj{x: s.0 - 5, y: (s.1 - 1), sprite: 'O'});
                    _time.total_time = updated_time;

                }else if t.as_secs() <= 0 && _enemies.len() == 0 {
                    _enemies.push(Obj{x: s.0 - 5, y: s.1 -1 , sprite: 'O'});
                }
            },
            Err(e) => panic!("Error{:?}",e),
        }

        },
        Err(e) => panic!("Error: {}",e),
    }
  true
}

fn drawenviorment(_x: &mut u16, _y: &mut u16, _stars: &Vec<(u16, u16)>, _terminal_coords: &Result<(u16, u16), Error>){
    let mut stdout = stdout().into_raw_mode().unwrap();
    let mut land = "=".to_owned();
    match _terminal_coords{
        Ok(d) =>{
            for _i in 0..d.0 -1{
                land.push_str("=");
            }
        },
        Err(e) => panic!("Err: {}",e),
    }
    write!(stdout,"{}{}{}",termion::cursor::Hide,termion::cursor::Goto(0, *_y + 1),land).unwrap();

    for i in 0.._stars.len(){
        write!(stdout,"{}{}*",termion::cursor::Hide, termion::cursor::Goto(_stars[i].0,_stars[i].1)).unwrap();
    }

}

fn drawenemes(_enemies: &Vec<Obj>) {
    let mut stdout = stdout().into_raw_mode().unwrap();
    for enemy in _enemies{
         write!(stdout,"{}{}{}{}{}",
                   termion::cursor::Hide,
                   termion::cursor::Goto(enemy.x, enemy.y),
                   enemy.sprite,
                   termion::cursor::Goto(enemy.x, enemy.y - 1),
                   enemy.sprite).unwrap();
    }

}


// helper fib fuunction to generat a nice background.
fn generatebackground(_terminal_coords: &mut Result<(u16,u16),Error>) -> Vec<(u16,u16)>{
    // object used to return from function
    let mut vec: Vec<(u16,u16)> = Vec::new();
    match _terminal_coords{
        Ok(d) =>{

           for _i in 0..d.0{
               let p1 = rand::thread_rng().gen_range(0,d.0 / 2);
               let p2 = rand::thread_rng().gen_range(d.0 /2,d.0);
               let p3 = rand::thread_rng().gen_range(0,(d.1 - 4) / 2);
               let p4 = rand::thread_rng().gen_range((d.1 -4) /2, d.1);

               for _i in 0..rand::thread_rng().gen_range(1, 5){
                   vec.push((rand::thread_rng().gen_range(p1, p2), rand::thread_rng().gen_range(p3,p4)));
               }
           }
           vec
        },
        Err(e) => panic!("Error: {}",e),
    }
}
fn main() {
    //need to check if tty is supported on the current Running OS. 
    if !termion::is_tty(&fs::File::create("/dev/stdout").unwrap()) {
        println!("This is not a TTY :(");
        return;
    }


    //get the size of the current terminal emulator
    let mut terminal_dimension = termion::terminal_size();
    // Get the standard output stream and go to raw mode.
    let mut stdout = stdout().into_raw_mode().unwrap();
    // Get the standard input stream.
    let stdin = async_stdin();
    let mut bytes = stdin.bytes();

    // player object intilization ( garbage coord values until initlization).
    let mut player = Player{x: 4, y: 20u16, jump: false, score: 0, sprite: 'P'};

    let mut env_x;
    let mut env_y;

    match terminal_dimension{
        Ok(d) => {
            player.x = d.0 / 24;
            player.y = d.1 -1;
            env_x = d.0 / 24;
            env_y = d.1;
        },
        Err(e) => panic!("cannot get terminal dimensions {}",e),
    }

    let mut enemies: Vec<Obj> = Vec::new();

    let mut stars: Vec<(u16,u16)> = generatebackground(&mut terminal_dimension);

    // start the timer for the game clock.
    let timer = SystemTime::now();

    //Construct our time object and add it to our container.
    let mut times = Time{total_time: timer};

    //Game loop
    loop {
       let b = bytes.next();
        if !update(&mut player, &mut enemies,&mut stars, &mut times, &mut terminal_dimension){
            break;
        }

       if let Some(Ok(b'q')) = b {
            break;
       }

        match terminal_dimension{
            Ok(d) => {
                if let Some(Ok(b'w')) = b {
                    if player.y >= (d.1 - 1) {
                        player.jump = true;
                    }
                }

                if player.jump{
                    playerjump(&mut player, &mut terminal_dimension);

                    if player.y <= (d.1 - 1) - 4{
                        player.jump = false;
                    }
                }
            },
            Err(e) => panic!("cannot get terminal dimensions {}",e),
       }
       // update the screen with updated position
       drawplayer(&mut player);
       drawenemes(&enemies);
       drawenviorment(&mut env_x, &mut env_y, &stars, &mut terminal_dimension);
       stdout.flush().unwrap();

       // Have the thread sleep in order to ensure as steady fps we sleep sleep for 1000/fps
       sleep(Duration::from_millis(1000/25 as  u64));
    }
    // Show the cursor again and clear the screen before we exit.
    write!(stdout, "{}{}{}Game Over score: {}\n{}",
           termion::cursor::Goto(1,1),
           termion::clear::All,
           termion::cursor::Show,
           player.score,
           termion::cursor::Goto(1,2)).unwrap();

}
