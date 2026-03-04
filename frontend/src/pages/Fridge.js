import { useState, useEffect, useCallback } from 'react';
import { Link } from 'react-router-dom';
import { useAuth } from '../context/AuthContext';
import api from '../api/axios';
import Toast from '../components/Toast';
import styles from './Fridge.module.css';

function Fridge() {
  const { user } = useAuth();
  const [fridge, setFridge] = useState([]);
  const [allIngredients, setAllIngredients] = useState([]);
  const [selectedId, setSelectedId] = useState('');
  const [recipes, setRecipes] = useState([]);
  const [loading, setLoading] = useState(true);
  const [searching, setSearching] = useState(false);
  const [adding, setAdding] = useState(false);
  const [toast, setToast] = useState(null);

  const fetchFridge = useCallback(async () => {
    try {
      const [fridgeRes, ingredientsRes] = await Promise.all([
        api.get('/fridge'),
        api.get('/ingredients'),
      ]);
      setFridge(Array.isArray(fridgeRes.data) ? fridgeRes.data : []);
      setAllIngredients(Array.isArray(ingredientsRes.data) ? ingredientsRes.data : []);
    } catch (err) {
      console.error('Fridge fetch error:', err);
      setFridge([]);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    if (user) {
      fetchFridge();
    } else {
      setLoading(false);
    }
  }, [user, fetchFridge]);

  const addToFridge = useCallback(async (e) => {
    if (e) {
      e.preventDefault();
      e.stopPropagation();
    }
    if (!selectedId || adding) return;
    setAdding(true);
    try {
      const addedName = allIngredients.find((i) => i.id === selectedId)?.name || 'Sastojak';
      await api.post('/fridge', { ingredient_id: selectedId });
      setSelectedId('');
      setToast({ message: `${addedName} dodan u frižider!`, type: 'success' });
      await fetchFridge();
    } catch (err) {
      console.error('Add to fridge error:', err.response?.status, err.response?.data);
      setToast({ message: err.response?.data?.error || 'Greška pri dodavanju', type: 'error' });
    } finally {
      setAdding(false);
    }
  }, [selectedId, adding, allIngredients, fetchFridge]);

  const removeFromFridge = async (ingredientId) => {
    try {
      const removedName = fridge.find((i) => i.id === ingredientId)?.name || 'Sastojak';
      await api.post('/fridge/remove', { ingredient_id: ingredientId });
      setToast({ message: `${removedName} uklonjen iz frižidera`, type: 'info' });
      await fetchFridge();
      setRecipes([]);
    } catch (err) {
      console.error('Remove error:', err.response?.status, err.response?.data);
      setToast({ message: err.response?.data?.error || 'Greška pri uklanjanju', type: 'error' });
    }
  };

  const findRecipes = async (e) => {
    if (e) {
      e.preventDefault();
      e.stopPropagation();
    }
    if (fridge.length === 0) {
      setToast({ message: 'Dodajte sastojke u frižider', type: 'error' });
      return;
    }
    setSearching(true);
    try {
      const res = await api.post('/fridge/recipes', {});
      const data = Array.isArray(res.data) ? res.data : [];
      setRecipes(data);
      if (data.length === 0) {
        setToast({ message: 'Nema recepata sa ovim sastojcima', type: 'info' });
      } else {
        setToast({ message: `Pronađeno ${data.length} recepata!`, type: 'success' });
      }
    } catch (err) {
      console.error('Find recipes error:', err.response?.status, err.response?.data);
      setToast({ message: err.response?.data?.error || 'Greška pri pretrazi', type: 'error' });
    } finally {
      setSearching(false);
    }
  };

  if (!user) {
    return (
      <div className={styles.page}>
        <div className={styles.empty}>
          <span className={styles.emptyIcon}>🔒</span>
          <h3>Prijavite se</h3>
          <p>Morate biti prijavljeni da koristite frižider</p>
          <Link to="/login" className={styles.loginLink}>Prijava</Link>
        </div>
      </div>
    );
  }

  if (loading) {
    return (
      <div className={styles.loading}>
        <div className={styles.spinner}></div>
        <p>Učitavanje frižidera...</p>
      </div>
    );
  }

  const available = allIngredients.filter(
    (ing) => !fridge.find((f) => f.id === ing.id)
  );

  const totalCalories = fridge.reduce((sum, item) => sum + (item.calories_per_100g || 0), 0);

  return (
    <div className={styles.page}>
      {toast && <Toast message={toast.message} type={toast.type} onClose={() => setToast(null)} />}

      <div className={styles.header}>
        <h1>🧊 Moj Frižider</h1>
        <p className={styles.subtitle}>{fridge.length} sastojaka</p>
      </div>

      <div className={styles.addSection} onClick={(e) => e.stopPropagation()}>
        <select
          value={selectedId}
          onChange={(e) => {
            e.stopPropagation();
            setSelectedId(e.target.value);
          }}
          className={styles.select}
        >
          <option value="">Izaberi sastojak za dodati...</option>
          {available.map((ing) => (
            <option key={ing.id} value={ing.id}>
              {ing.name} ({ing.calories_per_100g} kcal/100g)
            </option>
          ))}
        </select>
        <button
          type="button"
          onClick={addToFridge}
          className={styles.addBtn}
          disabled={!selectedId || adding}
        >
          {adding ? 'Dodajem...' : '+ Dodaj'}
        </button>
      </div>

      {fridge.length === 0 ? (
        <div className={styles.empty}>
          <span className={styles.emptyIcon}>🧊</span>
          <h3>Frižider je prazan</h3>
          <p>Dodajte sastojke da pronađete recepte</p>
        </div>
      ) : (
        <>
          <div className={styles.items}>
            {fridge.map((item, i) => (
              <div key={item.id} className={styles.item} style={{ animationDelay: `${i * 0.05}s` }}>
                <div className={styles.itemInfo}>
                  <span className={styles.itemName}>{item.name}</span>
                  <div className={styles.itemStats}>
                    <span className={styles.stat}>🔥 {item.calories_per_100g} kcal</span>
                    <span className={styles.stat}>💪 {item.protein_per_100g}g protein</span>
                  </div>
                </div>
                <button
                  type="button"
                  onClick={(e) => {
                    e.preventDefault();
                    e.stopPropagation();
                    removeFromFridge(item.id);
                  }}
                  className={styles.removeBtn}
                >
                  ✕
                </button>
              </div>
            ))}
          </div>

          <div className={styles.summary}>
            <span>Ukupno kalorija (po 100g svakog): <strong>{totalCalories.toFixed(0)} kcal</strong></span>
          </div>

          <button
            type="button"
            onClick={findRecipes}
            className={styles.findBtn}
            disabled={searching}
          >
            {searching ? 'Tražim...' : '🔍 Pronađi recepte sa mojim sastojcima'}
          </button>
        </>
      )}

      {recipes.length > 0 && (
        <div className={styles.results}>
          <h2>🍳 Recepti koje možeš napraviti ({recipes.length})</h2>
          <div className={styles.recipeGrid}>
            {recipes.map((recipe) => (
              <Link to={`/recipes/${recipe.id}`} key={recipe.id} className={styles.recipeCard}>
                <h3>{recipe.title}</h3>
                <p>{recipe.description}</p>
                <div className={styles.recipeMeta}>
                  <span>👤 {recipe.username}</span>
                  <span>⏱️ {(recipe.prep_time_min || 0) + (recipe.cook_time_min || 0)} min</span>
                </div>
              </Link>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}

export default Fridge;