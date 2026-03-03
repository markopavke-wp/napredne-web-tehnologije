CREATE TABLE users (
    id UUID PRIMARY KEY,
    username VARCHAR(50) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE ingredients (
    id UUID PRIMARY KEY,
    name VARCHAR(100) UNIQUE NOT NULL,
    calories_per_100g DOUBLE PRECISION NOT NULL DEFAULT 0,
    protein_per_100g DOUBLE PRECISION NOT NULL DEFAULT 0,
    carbs_per_100g DOUBLE PRECISION NOT NULL DEFAULT 0,
    fat_per_100g DOUBLE PRECISION NOT NULL DEFAULT 0,
    fiber_per_100g DOUBLE PRECISION NOT NULL DEFAULT 0
);

CREATE TABLE recipes (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title VARCHAR(255) NOT NULL,
    description TEXT,
    instructions TEXT NOT NULL,
    prep_time_min INTEGER,
    cook_time_min INTEGER,
    servings INTEGER,
    image_url TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE recipe_ingredients (
    recipe_id UUID NOT NULL REFERENCES recipes(id) ON DELETE CASCADE,
    ingredient_id UUID NOT NULL REFERENCES ingredients(id) ON DELETE CASCADE,
    quantity DOUBLE PRECISION NOT NULL,
    unit VARCHAR(30) NOT NULL,
    PRIMARY KEY (recipe_id, ingredient_id)
);

CREATE INDEX idx_recipes_user_id ON recipes(user_id);
CREATE INDEX idx_recipes_created_at ON recipes(created_at DESC);