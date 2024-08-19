// Define the game configuration using the turbo::cfg! macro
turbo::cfg! {r#"
    name = "Breakout"
    version = "1.0.0"
    author = "Turbo"
    description = "Blow up some bricks"
    [settings]
    resolution = [128, 128]
"#}

turbo::init! {
    // Define the GameState struct.
    struct GameState {
        screen: enum Screen {
            Title,
            Level,
        },
        ball_x: f32,
        ball_y: f32,
        ball_dx: f32,
        ball_dy: f32,
        ball_d: f32,
        pad_dx: f32,
        pad_x: f32,
        pad_y: f32,
        pad_w: f32,
        pad_h: f32,
        mode: String,
        brick_x: Vec<f32>,
        brick_y: Vec<f32>,
        brick_v: Vec<bool>,
        brick_t: Vec<String>,
        brick_w: f32,
        brick_h: f32,
        lives: i32,
        points: i32,
        brick_hit: bool,
        sticky: bool,
        ball_ang: f32,
        chain: i32,
        levels: Vec<String>,
        levelnum: usize,
        level: String,
    } = {
        // Set the struct's initial value.
        Self {
            screen: Screen::Title,
            ball_x: 0.0,
            ball_y: 66.0,
            ball_dx: 1.0,
            ball_dy: 1.0,
            ball_d: 5.5,
            pad_dx: 0.0,
            pad_x: 52.0,
            pad_y: 120.0,
            pad_w: 24.0,
            pad_h: 4.0,
            mode: "start".to_string(),
            lives: 3,
            points: 0,
            brick_x: vec![],
            brick_y: vec![],
            brick_v: vec![],
            brick_w: 10.0,
            brick_h: 4.0,
            brick_hit: false,
            sticky: true,
            ball_ang: 1.0,
            chain: 1,
            levelnum: 0,
            levels: vec![
                "hxixsxpxbxbxbxbxbxbxbxbxbx".to_string(),
                // "bbbbb".to_string(),
                "x6b".to_string(),
                // "x7b".to_string(),
            ],
            level: "".to_string(),
            brick_t: vec![
                "x7b".to_string(),
                "x6b".to_string(),
            ],
        }
    }
}

// This is where your main game loop code goes
// The stuff in this block will run ~60x per sec
turbo::go! { 
    let mut state = GameState::load();

    state.mode = "start".to_string();

    state.level = state.levels[state.levelnum as usize].clone();
    
    _update60();
    _draw();
}

fn _update60 () {
    let mut state = GameState::load();

    if state.mode == "game" {
        update_game()
    } 
    if state.mode == "start" {
        update_start()
    }
    if state.mode == "gameover" {
        update_gameover()
    }
    if state.mode == "levelover" {
        update_levelover()
    }
}

fn update_levelover() {
    draw_gameover();
    let gp = gamepad(0);
    if gp.start.just_pressed() {
        // next level
        nextlevel();
    }
}

fn update_start() {
    let gp = gamepad(0);
    if gp.start.just_pressed() {
        startgame();
    }
}

fn startgame () {
    let mut state = GameState::load();
    state.pad_x=52.0;
    state.pad_y=120.0;
    state.pad_dx=0.0;
    state.pad_w=24.0;
    state.pad_h=3.0;
    state.lives=3;
    state.points = 0;
    state.levelnum = 0;

    state.mode = "game".to_string();
    state.save();
    buildbricks();
    serveball();
}

fn nextlevel () {
    buildbricks();
    let mut state = GameState::load();
    state.pad_x=52.0;
    state.pad_y=120.0;
    state.pad_dx=0.0;
    state.pad_w=24.0;
    state.pad_h=3.0;
    state.levelnum += 1;
    if state.levelnum > state.levels.len() - 1 {
        //game over here
        state.mode = "start".to_string();
        state.levelnum = 0;
    } else {
        state.mode = "game".to_string();
    }
    state.save();
    buildbricks();
    serveball();
}

fn buildbricks() {
    let mut state = GameState::load();
    state.brick_x = vec![];
    state.brick_y = vec![];
    state.brick_v = vec![];
    state.brick_t = vec![];

    let mut lvl = state.levels[state.levelnum].chars().collect::<Vec<char>>();
    let mut j = 0.0;
    let mut o = 0.0;
    let mut last = "".to_string();

    for i in 0..lvl.len()  {
        j+=1.0;
        let char = lvl[i];
        if char == 'b' || char == 'i' || char == 'h' || char == 's' || char == 'p' {
            last = char.to_string();
            state = addbrick(state, j, char.to_string());
        } else if char == 'x' {
            last="x".to_string();
        } else if char == '/' {
            j = (((j - 1.0) / 11.0)+1.0).floor() * 11.0;
        } else if char >= '0' && char <= '9' {
            for _ in 0..char.to_digit(10).unwrap() {
                if last == "b" || last == "i" || last == "h" || last == "s" || last == "p" {
                    state = addbrick(state, j, char.to_string());
                } else if last == "x" {
                    //nothing
                }
                j+=1.0;
            }
            j-=1.0
        }
        state.save();
    }
}

fn addbrick(mut state: GameState, _i: f32, _t: String) -> GameState {
    state.brick_x.push(4.0 + (((_i - 1.0) % 11.0) * (state.brick_w + 1.0)));
    state.brick_y.push(20.0 + ((_i - 1.0) / 11.0).floor() * (state.brick_h + 1.0));
    state.brick_v.push(true);
    state.brick_t.push(_t);
    state
}

fn levelfinished(state: &GameState) -> bool {
    if state.brick_v.is_empty() {
        return true; // If there are no bricks, the level is finished
    }
    for &brick_visible in &state.brick_v {
        if brick_visible {
            return false; // If any brick is still visible, the level is not finished
        }
    }
    true // All bricks are invisible, so the level is finished
}

fn serveball() {
    let mut state = GameState::load();
    state.ball_x = (state.pad_x + state.pad_w / 2.0) - (state.ball_d * 0.5);
    state.ball_y = state.pad_y - (state.ball_d * 0.9);
    state.ball_dx = state.ball_dx.abs(); 
    state.ball_dy = state.ball_dy.abs();
    state.sticky = true;
    state.chain = 1;
    state.save();
}

fn gameover() {
    let mut state = GameState::load();
    state.mode = "gameover".to_string();
    state.save();
}

fn levelover() -> String {
    "levelover".to_string()
}

fn update_gameover() {
    draw_gameover();
    let gp = gamepad(0);
    if gp.start.just_pressed() {
        startgame();
    }
}

fn update_game() {
    let mut state = GameState::load();

    let gp = gamepad(0);

    let mut buttpress = false;

    //move the paddle
    if gp.right.pressed() {
        state.pad_dx = 2.5;
        buttpress = true;
        if state.sticky {
            state.ball_dx = 1.0;
        }
    }
    if gp.left.pressed() {
        state.pad_dx = -2.5;
        buttpress = true;
        if state.sticky {
            state.ball_dx = -1.0;
        }
    }

    if !buttpress {
        state.pad_dx = state.pad_dx / 2.3;
    }


    state.pad_x += state.pad_dx;
    state.pad_x = mid(-1.0, state.pad_x, 128.0 - state.pad_w);


    if state.sticky && gp.start.just_pressed() {
        state.sticky = false;
    }

    state.save();   

    if state.sticky {
        state.ball_x = (state.pad_x + state.pad_w / 2.0) - (state.ball_d / 2.0);
        state.ball_y = state.pad_y - (state.ball_d * 0.9);
        state.save();   
    } else {
        let mut nextx;
        let mut nexty;

        nextx = state.ball_x + state.ball_dx;
        nexty = state.ball_y + state.ball_dy;

        //ball hits side
        if nextx > 123.0 || nextx < 0.0 {
            nextx = mid(0.0, nextx, 127.0);
            state.ball_dx = -1.0 * state.ball_dx;
        }
        if nexty < 7.0 {
            nexty = mid(0.0, nexty, 127.0);
            state.ball_dy = -1.0 * state.ball_dy;
        }

        // check if ball hits pad
        if ball_box(nextx, nexty, state.ball_d, state.pad_x, state.pad_y, state.pad_w, state.pad_h) {
            //deal with collision
            //find out in which direction to deflect
            if deflx_ball_box(state.ball_x, state.ball_y, state.ball_dx, state.ball_dy, state.pad_x, state.pad_y, state.pad_w, state.pad_h) {
                // ball hit paddle on the side
                state.ball_dx = -state.ball_dx;
                if state.ball_x < (state.pad_x + state.pad_w / 2.0 ){
                    nextx = state.pad_x - state.ball_d;
                } else {
                    nextx = state.pad_x + state.pad_w + state.ball_d;
                }
            } else {
                // ball hit paddle on the top/bottom
                state.ball_dy = -state.ball_dy;
                if state.ball_y > state.pad_y {
                    //bottom
                    nexty = state.pad_y + state.pad_h + state.ball_d;
                } else {
                    //top
                    nexty = state.pad_y - state.ball_d;
                    if state.pad_dx.abs() > 2.0 {
                        // change angle
                        if state.pad_dx.signum() == state.ball_dx.signum() {
                            //decrease angle
                            let mut ang = mid(0.0,state.ball_ang-1.0,2.0);
                            state.ball_ang = ang;
                            if ang == 2.0 {
                                state.ball_dx = 0.50 * state.ball_dx.signum();
                                state.ball_dy = 1.30 * state.ball_dy.signum();
                            } else if ang == 0.0 {
                                state.ball_dx = 1.30 * state.ball_dx.signum();
                                state.ball_dy = 0.50 * state.ball_dy.signum();
                            } else {
                                state.ball_dx = 1.0 * state.ball_dx.signum();
                                state.ball_dy = 1.0 * state.ball_dy.signum();
                            }
                        } else {
                            //increase angle
                            if state.ball_ang == 2.0 {
                                state.ball_dx = -1.0 * state.ball_dx;
                            } else {
                                let mut ang = mid(0.0,state.ball_ang+1.0,2.0);
                                state.ball_ang = ang;
                                if ang == 2.0 {
                                    state.ball_dx = 0.50 * state.ball_dx.signum();
                                    state.ball_dy = 1.30 * state.ball_dy.signum();
                                } else if ang == 0.0 {
                                    state.ball_dx = 1.30 * state.ball_dx.signum();
                                    state.ball_dy = 0.50 * state.ball_dy.signum();
                                } else {
                                    state.ball_dx = 1.0 * state.ball_dx.signum();
                                    state.ball_dy = 1.0 * state.ball_dy.signum();
                                }
                            }

                        }
                    }
                }
            }
            state.chain = 1;
        } else {
        }

        state.brick_hit = false;
        // check if ball hits brick
        for i in 0..state.brick_x.len() {
            if state.brick_v[i] && ball_box(nextx, nexty, state.ball_d, state.brick_x[i], state.brick_y[i], state.brick_w, state.brick_h) {
                //deal with collision
                //find out in which direction to deflect
                if !state.brick_hit {
                    if deflx_ball_box(state.ball_x, state.ball_y, state.ball_dx, state.ball_dy, state.brick_x[i], state.brick_y[i], state.brick_w, state.brick_h) {
                        state.ball_dx = -state.ball_dx;
                    } else {
                        state.ball_dy = -state.ball_dy;
                    }
                }
                state.brick_hit = true;
                hitbrick(&mut state, i);
                if levelfinished(&state) {
                    state.mode = levelover();
                }
            }
        }

        //move the ball
        state.ball_x = nextx;
        state.ball_y = nexty;

        state.save();

        if nexty > 127.0 {
            state.lives -= 1;
            if state.lives < 0 {
                state.save();
                gameover();
            } else {
                state.save();
                serveball();
            }
        }
    }
}

fn hitbrick(state: &mut GameState, i: usize) {
    if state.brick_t[i] == "b" {
        state.brick_v[i] = false;
        state.points += (10 * state.chain);
        state.chain += 1;
        state.chain = state.chain.max(1).min(7);
        state.save(); // Save the state after modifications
    }
}

fn _draw() {
    let state = GameState::load();

    if state.mode == "game" {
        draw_game()
    } 
    if state.mode == "start" {
        draw_start()
    }
    if state.mode == "gameover" {
        draw_gameover()
    }
    if state.mode == "levelover" {
        draw_levelover()
    }
}

fn draw_start() {
    clear!();
    text!("TURBO BREAKOUT", x = 30, y = 40, color = 0xFFF1E8, font = Font::M);
    text!("PRESS SPACE", x = 40, y = 70, color = 0xFFCCAA, font = Font::S);
}

fn draw_game() {
    //draw game 
    let mut state = GameState::load();
    let mut brickcol: u32 = 0xFF77A8;

    clear(0x1D2B53);

    if state.sticky {
        path!(
            start = ((state.ball_x + (state.ball_d / 2.0) + (state.ball_dx * 4.0)), (state.ball_y + (state.ball_d / 2.0) + (state.ball_dy * -4.0))),
            end = ((state.ball_x + (state.ball_d / 2.0) + (state.ball_dx * 7.0)), (state.ball_y + (state.ball_d / 2.0) + (state.ball_dy * -7.0))),
            width = 1,
            color = 0xFFF1E8,
        )
    }

    circ!(d = state.ball_d, 
        x = state.ball_x, 
        y = state.ball_y, 
        color = 0xFFF1E8);
    rect!(w = state.pad_w, 
        h = state.pad_h, 
        x = state.pad_x, 
        y = state.pad_y, 
        color = 0xFFF1E8);


    //draw bricks
    for i in 0..state.brick_v.len() {
        if (state.brick_v[i]) {
            if state.brick_t[i] == "b" {
                brickcol = 0xFF77A8;
            } else if state.brick_t[i] == "i" {
                brickcol = 0x5F574F;
            } else if state.brick_t[i] == "h" {
                brickcol = 0xC2C3C7;
            } else if state.brick_t[i] == "s" {
                brickcol = 0xFF00FF;
            } else if state.brick_t[i] == "p" {
                brickcol = 0x87CEFA;
            }
            rect!(w = state.brick_w,
                h = state.brick_h,
                x = state.brick_x[i],
                y = state.brick_y[i],
                color = brickcol
            );
        }
    }

    rect!(w = 128, h = 6, x = 0, y = 0, color = 0x000000fff);
    text!(format!("Lives: {}", state.lives).as_str(), x = 1, y = 1, color = 0x00ff00ff, font = Font::S);
    text!(format!("Points: {}", state.points).as_str(), x = 73, y = 1, color = 0x00ff00ff, font = Font::S);
}

fn draw_gameover() {
    draw_game(); 
    rect!(w = 128, h = 15, x = 0, y = 60, color = 0x000000ff);
    text!("GAME OVER", x = 46, y = 62, color = 0xFFFFFF, font = Font::S);
    text!("PRESS SPACE TO RESTART", x = 15, y = 68, color = 0xB0B0B0, font = Font::S);
}

fn draw_levelover() {
    draw_game(); 
    rect!(w = 128, h = 15, x = 0, y = 60, color = 0x000000ff);
    text!("STAGE CLEAR!", x = 39, y = 62, color = 0xFFFFFF, font = Font::S);
    text!("PRESS SPACE TO CONTINUE", x = 8, y = 68, color = 0xD3D3D3, font = Font::S);
}

fn ball_box(bx: f32, by: f32, ball_d: f32, box_x: f32, box_y: f32, box_w: f32, box_h: f32) -> bool {
    if by - (ball_d * 0.1) > box_y + box_h {
        return false;
    }
    else if by + (ball_d * 0.5) < box_y {
        return false;
    }
    else if bx > box_x + box_w {
        return false;
    }
    else if bx + (ball_d * 0.5) < box_x {
        return false;
    } else {
        return true
    }
}

fn deflx_ball_box(ball_x: f32, ball_y: f32, ball_dx: f32, ball_dy: f32, box_x: f32, box_y: f32, box_w: f32, box_h: f32) -> bool {
    // calculate whether to reflect horizontally or vertically
    if ball_dx == 0.0 {
        //ball moving vertically
        return false;
    }
    else if ball_dy == 0.0 {
        //ball moving horizontally
        return true;
    }

    else {
        // moving diagonally
        let mut slp = ball_dy / ball_dx;
        let mut cx;
        let mut cy;

        if slp > 0.0 && ball_dx > 0.0 {
            // moving down and to the right
            cx = box_x - ball_x;
            cy = box_y - ball_y;
            if cx <= 0.0 {
                return false
            }
            else if cy/cx < slp {
                return true
            }
            else {
                return false
            }
        }
        else if slp < 0.0 && ball_dx > 0.0 {
            //moving up and to the right
            cx = box_x - ball_x;
            cy = box_y + box_h - ball_y;

            if cx <= 0.0 {
                return false
            }
            else if cy/cx < slp {
                return false
            }
            else {
                return true
            }
        }
        else if slp > 0.0 && ball_dx < 0.0 {
            // moving left and up
            cx = box_x + box_w - ball_x;
            cy = box_y + box_h - ball_y;
            if cx >= 0.0 {
                return false
            }
            else if cy/cx > slp {
                return false
            }
            else {
                return true
            }
        }
        else {
            // moving left and down
            cx = box_x + box_w - ball_x;
            cy = box_y - ball_y;
            if cx >= 0.0 {
                return false
            }
            else if cy/cx < slp {
                return false
            }
            else {
                return true
            }
        }
    }
    return false;
}

fn mid(min: f32, value: f32, max: f32) -> f32 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}