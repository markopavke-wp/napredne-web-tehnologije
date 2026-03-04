import { useState, useEffect } from 'react';
import { Link } from 'react-router-dom';
import api from '../api/axios';
import styles from './Search.module.css';

function Search() {
  const [ingredients, setIngredients] = useState([]);
  const [selectedIds, setSelectedIds] = useState([]);
  const [maxCalories, setMaxCalories] = useState('');
  const [maxTime, setMaxTime] = useState('');
  const [recipes, setRecipes] = useState([]);
  const [allRecipes, setAllRecipes] = useState([]);
  const [stats, setStats] = useState({});
  const [loading, setLoading] = useState(false);
  const [searched, setSearched] = useState(false);

  useEffect(() => {
    api.get('/ingredients').then((res) => setIngredients(res.data));
  }, []);

  const toggleIngredient = (id) => {
    setSelectedIds((prev) =>
      prev.includes(id) ? prev.filter((i) => i !== id) : [...prev, id]
    );
  };

  const handleSearch = async () => {
    setLoading(true);
    setSearched(true);
    try {
      let results;

      if (selectedIds.length > 0) {
        const res = await api.post('/recipes/search', { ingredient_ids: selectedIds });
        results = res.data;
      } else {
        const res = await api.get('/recipes');
        results = res.data;
      }

      // Filter po kalorijama - koristi ingredient podatke
      if (maxCalories) {
        const maxCal = Number(maxCalories);
        // Za sada filtriramo po vremenu jer nemamo total calories na receptu
        results = results.filter(() => true); // placeholder - treba backend endpoint
      }

      // Filter po vremenu pripreme
      if (maxTime) {
        const max = Number(maxTime);
        results = results.filter((r) => r.prep_time_min + r.cook_time_min <= max);
      }

      setAllRecipes(results);

      // Fetch stats za svaki recept
      const statsMap = {};
      await Promise.all(
        results.map(async (r) => {
          try {
            const s = await api.get(`/recipes/${r.id}/stats`);
            statsMap[r.id] = s.data;
          } catch {}
        })
      );
      setStats(statsMap);
      setRecipes(results);
    } catch (err) {
      console.error(err);
    } finally {
      setLoading(false);
    }
  };

  const clearFilters = () => {
    setSelectedIds([]);
    setMaxCalories('');
    setMaxTime('');
    setRecipes([]);
    setSearched(false);
  };

  return (
    <div className={styles.page}>
      <h1 className={styles.title}>🔍 Pretraga Recepata</h1>

      <div className={styles.filters}>
        <div className={styles.filterSection}>
          <h3>🥗 Filtriraj po sastojcima</h3>
          <p className={styles.filterHint}>Klikni na sastojke koje želiš u receptu</p>
          <div className={styles.ingredientGrid}>
            {ingredients.map((ing) => (
              <button
                key={ing.id}
                onClick={() => toggleIngredient(ing.id)}
                className={`${styles.ingredientChip} ${selectedIds.includes(ing.id) ? styles.selected : ''}`}
              >
                {ing.name}
                <span className={styles.chipCalories}>{ing.calories_per_100g} kcal</span>
              </button>
            ))}
          </div>
        </div>

        <div className={styles.filterRow}>
          <div className={styles.filterField}>
            <label>⏱️ Max vrijeme (min)</label>
            <input
              type="number"
              placeholder="npr. 30"
              value={maxTime}
              onChange={(e) => setMaxTime(e.target.value)}
              min={0}
            />
          </div>
          <div className={styles.filterField}>
            <label>🔥 Max kalorije (kcal)</label>
            <input
              type="number"
              placeholder="npr. 500"
              value={maxCalories}
              onChange={(e) => setMaxCalories(e.target.value)}
              min={0}
            />
          </div>
        </div>

        <div className={styles.filterActions}>
          <button onClick={handleSearch} className={styles.searchBtn} disabled={loading}>
            {loading ? 'Tražim...' : '🔍 Pretraži'}
          </button>
          <button onClick={clearFilters} className={styles.clearBtn}>Očisti filtere</button>
        </div>
      </div>

      {searched && (
        <div className={styles.results}>
          <h2>Rezultati ({recipes.length})</h2>
          {recipes.length === 0 ? (
            <div className={styles.noResults}>
              <span>😔</span>
              <p>Nema recepata koji odgovaraju filterima</p>
            </div>
          ) : (
            <div className={styles.recipeGrid}>
              {recipes.map((recipe) => (
                <Link to={`/recipes/${recipe.id}`} key={recipe.id} className={styles.recipeCard}>
                  <h3>{recipe.title}</h3>
                  <p>{recipe.description}</p>
                  <div className={styles.recipeMeta}>
                    <span>👤 {recipe.username}</span>
                    <span>⏱️ {recipe.prep_time_min + recipe.cook_time_min} min</span>
                    <span>🍽️ {recipe.servings} porcija</span>
                    {stats[recipe.id] && (
                      <span>⭐ {stats[recipe.id].average_rating.toFixed(1)}</span>
                    )}
                  </div>
                </Link>
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  );
}

export default Search;