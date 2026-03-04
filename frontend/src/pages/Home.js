import { useState, useEffect } from 'react';
import { Link } from 'react-router-dom';
import api from '../api/axios';
import RecipeCard from '../components/RecipeCard';
import styles from './Home.module.css';

function Home() {
  const [recipes, setRecipes] = useState([]);
  const [stats, setStats] = useState({});
  const [loading, setLoading] = useState(true);
  const [sortBy, setSortBy] = useState('newest');

  useEffect(() => {
    const fetchRecipes = async () => {
      try {
        const res = await api.get('/recipes');
        const data = Array.isArray(res.data) ? res.data : [];
        setRecipes(data);

        const statsMap = {};
        await Promise.all(
          data.map(async (r) => {
            try {
              const s = await api.get(`/recipes/${r.id}/stats`);
              statsMap[r.id] = s.data;
            } catch {}
          })
        );
        setStats(statsMap);
      } catch (err) {
        console.error('Greška pri učitavanju recepata:', err);
      } finally {
        setLoading(false);
      }
    };
    fetchRecipes();
  }, []);

  const sortedRecipes = [...recipes].sort((a, b) => {
    if (sortBy === 'newest') return new Date(b.created_at) - new Date(a.created_at);
    if (sortBy === 'oldest') return new Date(a.created_at) - new Date(b.created_at);
    if (sortBy === 'rating') return (stats[b.id]?.average_rating || 0) - (stats[a.id]?.average_rating || 0);
    if (sortBy === 'time') return (a.prep_time_min + a.cook_time_min) - (b.prep_time_min + b.cook_time_min);
    return 0;
  });

  if (loading) {
    return (
      <div className={styles.loading}>
        <div className={styles.spinner}></div>
        <p>Učitavanje recepata...</p>
      </div>
    );
  }

  return (
    <div className={styles.page}>
      <div className={styles.header}>
        <div>
          <h1 className={styles.title}>Svi Recepti</h1>
          <p className={styles.subtitle}>{recipes.length} recepata dostupno</p>
        </div>
        <div className={styles.controls}>
          <select
            className={styles.sort}
            value={sortBy}
            onChange={(e) => setSortBy(e.target.value)}
          >
            <option value="newest">Najnoviji</option>
            <option value="oldest">Najstariji</option>
            <option value="rating">Najbolje ocijenjeni</option>
            <option value="time">Najbrži</option>
          </select>
        </div>
      </div>

      {recipes.length === 0 ? (
        <div className={styles.empty}>
          <span className={styles.emptyIcon}>📝</span>
          <h3>Nema recepata</h3>
          <p>Budi prvi koji će dodati recept!</p>
          <Link to="/create" className={styles.emptyBtn}>Kreiraj recept</Link>
        </div>
      ) : (
        <div className={styles.grid}>
          {sortedRecipes.map((recipe) => (
            <RecipeCard key={recipe.id} recipe={recipe} stats={stats[recipe.id]} />
          ))}
        </div>
      )}
    </div>
  );
}

export default Home;