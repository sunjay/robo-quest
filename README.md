# robo-quest

This game is written to explicitly target the
[GameShell](https://www.clockworkpi.com/).

## Usage

Use `DISPLAY_SCALE=n` for some `n >= 1` to make seeing the game easier.

```bash
$ DISPLAY_SCALE=2 cargo run
```

## Story

The game begins with a robot that has just turned on in the middle of a forest.
It awakens into an unfamiliar world with no knowledge of where it is. After
trying to move around a bit, the robot discovers that it can move around and
even jump. Trying to hold its jump allows the robot to hover slightly and reach
the ground slower.

The robot begins to explore hoping that it will find some information about
where it came from. From the forest areas it reaches a mountain which has a sign
at the entrance indicating that a town is on the other side. The robot decides
to go to the town to see if it can find more information. At the summit of the
mountain, the robot sees the city...completely decimated. But by what? Who could
have destroyed this city? The robot detects some movement in the city and
decides to go down and check it out.

As the robot looks around, it comes to an alleyway where a little girl has been
cornered by someone...

A robot! This robot looks a lot like him but is cornering this young girl and
about to attack.

Suddenly everything clicks. The evil responsible for destroying this city is THE
ROBOTS. Now, the robot must choose. Either it can join the other robots in
destroying the rest of the cities or it can take its role as the only hope for
all of these people being attacked.

## Levels

### Forest

- Level 1: Basic movement controls are learned
- Level 2: Practicing movement controls and hovering
- Level 3: More practicing and discovering mountain entrance

### Mountain

- Level 4: Climbing and jumping, robot sees itself in a panel of ice
- Level 5: The summit, robot sees the city in ruins
- Level 6: Robot decides to go down the mountain to the city

### City: Turning Point

- Level 7: Robot explores city, stumbles on alleyway with other robot and girl
  - Cutscene, decision point: Good OR Evil
- Level 8: Robot goes through the rest of the city
  - Good: must "save" (collect) people and rescue them from dangerous situations
    (fire) and evil robots
  - Evil: must "destroy" people and stop fighters from destroying robots
- Level 9: Robot finds evil robot garrison hideout
  - Good: must destroy evil robots that are guiding the hideout
  - Evil: must enter the garrison and help them destroy the resistance

### Evil Robot Hideout

- Level 10: More hideout
  - Good: must destroy evil robots that are guiding the hideout
  - Evil: must enter the garrison and help them destroy the resistance
- Level 11: More hideout
  - Good: must destroy evil robots that are guiding the hideout
  - Evil: must enter the garrison and help them destroy the resistance
- Level 12: Hideout commander
  - Good: must destroy the evil robot commander (Boss)
  - Evil: must stop the resistance forces from destroying the evil robot
    commander
