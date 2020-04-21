// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com

// Genetic Algorithm, Evolving Shakespeare

// Demonstration of using a genetic algorithm to perform a search

// setup()
//  # Step 1: The Population
//    # Create an empty population (an array or ArrayList)
//    # Fill it with DNA encoded objects (pick random values to start)

// draw()
//  # Step 1: Selection
//    # Create an empty mating pool (an empty ArrayList)
//    # For every member of the population, evaluate its fitness based on some criteria / function,
//      and add it to the mating pool in a manner consistant with its fitness, i.e. the more fit it
//      is the more times it appears in the mating pool, in order to be more likely picked for reproduction.

//  # Step 2: Reproduction Create a new empty population
//    # Fill the new population by executing the following steps:
//       1. Pick two "parent" objects from the mating pool.
//       2. Crossover -- create a "child" object by mating these two parents.
//       3. Mutation -- mutate the child's DNA based on a given probability.
//       4. Add the child object to the new population.
//    # Replace the old population with the new population
//
//   # Rinse and repeat
// Example 9-1: GA Shakespeare Simplified

use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).run();
}

// A type to describe a psuedo-DNA, i.e. genotype
//   Here, a virtual organism's DNA is an array of character.
//   Functionality:
//      -- convert DNA into a string
//      -- calculate DNA's "fitness"
//      -- mate DNA with another set of DNA
//      -- mutate DNA

#[derive(Clone)]
struct Dna {
    genes: Vec<char>, // The genetic sequence
    fitness: f32,
}

impl Dna {
    fn new(num: usize) -> Self {
        let mut genes = Vec::new();
        for _ in 0..num {
            genes.push(random_ascii());
        }
        Dna {
            genes,
            fitness: 0.05,
        }
    }

    fn get_phrase(&self) -> String {
        self.genes.iter().cloned().collect()
    }

    // Fitness function (returns floating point % of "correct" characters)
    fn calculate_fitness(&mut self, target: &String) {
        let mut score = 0;
        for i in 0..self.genes.len() {
            println!("self.genes[i] = {}  || traget = {}", self.genes[i], target.chars().nth(i).unwrap());
            if self.genes[i] == target.chars().nth(i).unwrap() {
                score += 1;
            }
        }

        self.fitness = score as f32 / target.len() as f32;
    }

    fn crossover(&self, partner: &Dna) -> Dna {
        // A new child
        let mut child = Dna::new(self.genes.len());
        let midpoint = random_range(0, self.genes.len()); // Pick a midpoint

        // Half from one, half from the other
        for i in 0..self.genes.len() {
            if i > midpoint {
                child.genes[i] = self.genes[i];
            } else {
                child.genes[i] = partner.genes[i];
            }
        }
        child
    }

    // Based on a mutation probability, picks a new random character
    fn mutate(&mut self, mutation_rate: f32) {
        for i in 0..self.genes.len() {
            if random_f32() < mutation_rate {
                self.genes[i] = random_ascii();
            }
        }
    }
}

struct Model {
    mutation_rate: f32,    // Mutation rate
    population: Vec<Dna>,  // Vector to hold the current population
    mating_pool: Vec<Dna>, // Vector which we will use for our "mating pool"
    target: String,        // Target phrase
}

fn model(app: &App) -> Model {
    app.new_window().size(800, 200).view(view).build().unwrap();
    let target = "To be or not to be.".to_string();
    let total_population = 150;

    Model {
        mutation_rate: 0.01,
        population: vec![Dna::new(target.len()); total_population],
        mating_pool: Vec::new(),
        target,
    }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    for i in 0..model.population.len() {
        model.population[i].calculate_fitness(&model.target);
    }

    model.mating_pool.clear(); 
    for i in 0..model.population.len() {
        let nnn = model.population[i].fitness as usize * 100; // Arbitrary multiplier, we can also use monte carlo method and pick two random numbers
        for _ in 0..nnn {                                       
            model.mating_pool.push(model.population[i].clone());
        }

        if app.elapsed_frames() == 0 {
            println!("fitness = {}", model.population[i].fitness);
        }
    }

    dbg!(model.mating_pool.len());

    for i in 0..model.population.len() {
        let a = random_range(0, model.mating_pool.len());
        let b = random_range(0, model.mating_pool.len());
        let partner_a = &model.mating_pool[a];
        let partner_b = &model.mating_pool[b];
        let mut child = partner_a.crossover(partner_b);
        child.mutate(model.mutation_rate);
        model.population[i] = child;
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    frame.clear(WHITE);

    let win = app.window_rect();
    let draw = app.draw();
    let mut everything = String::new();

    for i in 0..model.population.len() {
        everything = format!("{}{}", everything, model.population[i].get_phrase());
    }
/*
    draw.text(&everything)
        .color(BLACK)
        .left_justify()
        .align_text_top()
        .font_size(18)
        .x(20.0)
        .y(-160.0)
        .wh(win.wh());

*/
    // Write the result of our drawing to the window's frame.
    draw.to_frame(app, &frame).unwrap();
}
