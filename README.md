# Othello
A working Othello game.

## Purpose
This is my first hands-on experience with Rust, a language I have a great interest in learning.

## Scope
There is a playable board and a simple but functional GUI. Additionally, there is a agent that can play against a human or itself. The agent has its own thread so as not freeze up the UI. For now, the agent can either make random moves or use the Minimax algorithm.

### The Minimax Algorithm
The Minimax algorithm explores the implications of potential futures of moves, counter moves, counter-counter moves, and so forth. It makes an exhaustive search of the decision tree up to a user-defined search depth. At the final search depth, the board is evaluated and the optimal move chosen. Because of the symmetric zero-sum nature of this game, the opponent's evaluation is the inverse of the player's evaluation. Hence the name Minimax; we are maximizing our own gain while minimizing that of the opponent at all times.

#### Limitations
The Minimax algorithm is a strong, classic algorithm for playing deterministic and symmetric board games like Othello. Its main weakness, however, is that by its symmetric nature, it must assume the opponent is playing optimally by the same logic. Generally, this is not a problem, since there are few ways of playing a game like Othello well, but it’s good to be aware of nevertheless.

## To do
The implementation of the Minimax algorithm is relatively simple and can be refined significantly. For example, the strength of board positions, such as corners, and the strategic dominance of the center board are not currently considered. As of now, only the net sum of the player's disks minus those of the opponent is considered for the board evaluation. I intend to work on this in future revisions.

## Usage
The UI should be mostly self-explanatory. The depth sliders determine how many moves ahead the Minimax agent evaluates. A higher depth leads to better decision-making, but it also requires more time to compute. Be aware that setting the depth to 8 or higher may cause the agent to take a long time to make a move.

## Acknowledgements
Most of my understanding of Rust comes from reading [this guide](https://github.com/nrc/r4cppp). I find it an excellent start for someone who like me comes from a strong C++ background. It skips the basics and does a fantastic rundown of what Rust adds to the landscape of high-performance development.
