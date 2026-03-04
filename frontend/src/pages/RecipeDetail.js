import { useState, useEffect, useCallback } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { useAuth } from '../context/AuthContext';
import api from '../api/axios';
import StarRating from '../components/StarRating';
import Toast from '../components/Toast';
import styles from './RecipeDetail.module.css';

function RecipeDetail() {
  const { id } = useParams();
  const { user } = useAuth();
  const navigate = useNavigate();
  const [recipe, setRecipe] = useState(null);
  const [ingredients, setIngredients] = useState([]);
  const [stats, setStats] = useState(null);
  const [ratings, setRatings] = useState([]);
  const [score, setScore] = useState(0);
  const [comment, setComment] = useState('');
  const [loading, setLoading] = useState(true);
  const [toast, setToast] = useState(null);

  // Edit mode
  const [editing, setEditing] = useState(false);
  const [editTitle, setEditTitle] = useState('');
  const [editDescription, setEditDescription] = useState('');
  const [editInstructions, setEditInstructions] = useState('');
  const [editPrepTime, setEditPrepTime] = useState(0);
  const [editCookTime, setEditCookTime] = useState(0);
  const [editServings, setEditServings] = useState(1);

  // Edit ingredients
  const [editIngredients, setEditIngredients] = useState([]);
  const [allIngredients, setAllIngredients] = useState([]);

  const fetchData = useCallback(async () => {
    try {
      const recipeRes = await api.get(`/recipes/${id}`);
      const data = recipeRes.data;
      setRecipe(data);

      setEditTitle(data.title || '');
      setEditDescription(data.description || '');
      setEditInstructions(data.instructions || '');
      setEditPrepTime(data.prep_time_min || 0);
      setEditCookTime(data.cook_time_min || 0);
      setEditServings(data.servings || 1);

      // Dohvati sastojke recepta
      try {
        const ingRes = await api.get(`/recipes/${id}/ingredients`);
        const ingData = Array.isArray(ingRes.data) ? ingRes.data : [];
        setIngredients(ingData);
        setEditIngredients(ingData.map(ing => ({
          ingredient_id: ing.ingredient_id || ing.id,
          quantity: ing.quantity,
          unit: ing.unit,
        })));
      } catch {
        setIngredients([]);
      }

      try {
        const statsRes = await api.get(`/recipes/${id}/stats`);
        setStats(statsRes.data);
      } catch {}

      try {
        const ratingsRes = await api.get(`/recipes/${id}/ratings`);
        setRatings(Array.isArray(ratingsRes.data) ? ratingsRes.data : []);
      } catch {}
    } catch (err) {
      console.error(err);
    } finally {
      setLoading(false);
    }
  }, [id]);

  useEffect(() => { fetchData(); }, [fetchData]);

  // Dohvati sve sastojke kad se uđe u edit mode
  useEffect(() => {
    if (editing) {
      api.get('/ingredients')
        .then(res => setAllIngredients(res.data))
        .catch(console.error);
    }
  }, [editing]);

  const addEditIngredient = () => {
    setEditIngredients([...editIngredients, { ingredient_id: '', quantity: 100, unit: 'g' }]);
  };

  const updateEditIngredient = (index, field, value) => {
    const updated = [...editIngredients];
    updated[index][field] = field === 'quantity' ? Number(value) : value;
    setEditIngredients(updated);
  };

  const removeEditIngredient = (index) => {
    setEditIngredients(editIngredients.filter((_, i) => i !== index));
  };

  const handleRate = async (e) => {
    e.preventDefault();
    if (score === 0) {
      setToast({ message: 'Izaberite ocjenu (1-5 zvjezdica)', type: 'error' });
      return;
    }
    try {
      await api.post(`/recipes/${id}/rate`, { score, comment });
      setComment('');
      setScore(0);
      setToast({ message: `Ocjena ${score}/5 uspješno dodana!`, type: 'success' });
      fetchData();
    } catch (err) {
      setToast({ message: err.response?.data?.error || 'Greška pri ocjenjivanju', type: 'error' });
    }
  };

  const handleDelete = async () => {
    if (!window.confirm('Sigurno želite obrisati ovaj recept?')) return;
    try {
      await api.delete(`/recipes/${id}`);
      setToast({ message: 'Recept uspješno obrisan!', type: 'success' });
      setTimeout(() => navigate('/'), 1000);
    } catch (err) {
      setToast({ message: err.response?.data?.error || 'Greška pri brisanju', type: 'error' });
    }
  };

  const handleUpdate = async (e) => {
    e.preventDefault();
    try {
      await api.put(`/recipes/${id}`, {
        title: editTitle,
        description: editDescription,
        instructions: editInstructions,
        prep_time_min: Number(editPrepTime),
        cook_time_min: Number(editCookTime),
        servings: Number(editServings),
        ingredients: editIngredients.map(s => ({
          ingredient_id: s.ingredient_id,
          quantity: s.quantity,
          unit: s.unit,
        })),
      });
      setEditing(false);
      setToast({ message: 'Recept uspješno ažuriran!', type: 'success' });
      fetchData();
    } catch (err) {
      setToast({ message: err.response?.data?.error || 'Greška pri ažuriranju', type: 'error' });
    }
  };

  if (loading) {
    return (
      <div className={styles.loading}>
        <div className={styles.spinner}></div>
        <p>Učitavanje recepta...</p>
      </div>
    );
  }

  if (!recipe) {
    return (
      <div className={styles.notFound}>
        <span>😔</span>
        <h3>Recept nije pronađen</h3>
        <button onClick={() => navigate('/')} className={styles.backBtn}>← Nazad na recepte</button>
      </div>
    );
  }

  const isOwner = user && user.id === recipe.user_id;

  return (
    <div className={styles.page}>
      {toast && <Toast message={toast.message} type={toast.type} onClose={() => setToast(null)} />}

      <div className={styles.card}>
        <div className={styles.topBar}>
          <button type="button" onClick={() => navigate('/')} className={styles.backBtn}>← Nazad</button>
          {isOwner && (
            <div className={styles.ownerActions}>
              <button
                type="button"
                onClick={() => setEditing(!editing)}
                className={styles.editBtn}
              >
                {editing ? '✕ Otkaži' : '✏️ Uredi'}
              </button>
              <button type="button" onClick={handleDelete} className={styles.deleteBtn}>🗑️ Obriši</button>
            </div>
          )}
        </div>

        {editing ? (
          <form onSubmit={handleUpdate} className={styles.editForm}>
            <div className={styles.field}>
              <label>Naziv</label>
              <input type="text" value={editTitle} onChange={(e) => setEditTitle(e.target.value)} required />
            </div>
            <div className={styles.field}>
              <label>Opis</label>
              <textarea value={editDescription} onChange={(e) => setEditDescription(e.target.value)} required rows={2} />
            </div>
            <div className={styles.field}>
              <label>Upute za pripremu</label>
              <textarea value={editInstructions} onChange={(e) => setEditInstructions(e.target.value)} required rows={5} />
            </div>
            <div className={styles.editRow}>
              <div className={styles.field}>
                <label>⏱️ Priprema (min)</label>
                <input type="number" value={editPrepTime} onChange={(e) => setEditPrepTime(e.target.value)} min={0} />
              </div>
              <div className={styles.field}>
                <label>🔥 Kuvanje (min)</label>
                <input type="number" value={editCookTime} onChange={(e) => setEditCookTime(e.target.value)} min={0} />
              </div>
              <div className={styles.field}>
                <label>🍽️ Porcije</label>
                <input type="number" value={editServings} onChange={(e) => setEditServings(e.target.value)} min={1} />
              </div>
            </div>

            {/* Edit sastojci */}
            <div className={styles.ingredientsSection}>
              <div className={styles.ingredientsHeader}>
                <h3>🥗 Sastojci</h3>
                <button type="button" onClick={addEditIngredient} className={styles.addIngBtn}>+ Dodaj</button>
              </div>
              {editIngredients.length === 0 && (
                <p className={styles.noIngredients}>Kliknite "Dodaj" za dodavanje sastojaka</p>
              )}
              {editIngredients.map((ing, i) => (
                <div key={i} className={styles.ingredientEditRow}>
                  <select value={ing.ingredient_id} onChange={(e) => updateEditIngredient(i, 'ingredient_id', e.target.value)} required>
                    <option value="">Izaberi...</option>
                    {allIngredients.map((a) => (
                      <option key={a.id} value={a.id}>{a.name}</option>
                    ))}
                  </select>
                  <input type="number" placeholder="Količina" value={ing.quantity} onChange={(e) => updateEditIngredient(i, 'quantity', e.target.value)} min={1} />
                  <select value={ing.unit} onChange={(e) => updateEditIngredient(i, 'unit', e.target.value)}>
                    <option value="g">g</option>
                    <option value="ml">ml</option>
                    <option value="kom">kom</option>
                    <option value="kasika">kašika</option>
                    <option value="kasicica">kašičica</option>
                  </select>
                  <button type="button" onClick={() => removeEditIngredient(i)} className={styles.removeIngBtn}>✕</button>
                </div>
              ))}
            </div>

            <button type="submit" className={styles.saveBtn}>💾 Sačuvaj izmjene</button>
          </form>
        ) : (
          <>
            <h1 className={styles.title}>{recipe.title}</h1>
            <div className={styles.authorRow}>
              <span className={styles.author}>👤 {recipe.username}</span>
              {stats && (
                <div className={styles.statsRow}>
                  <StarRating rating={Math.round(stats.average_rating || 0)} readonly size={20} />
                  <span className={styles.statsText}>
                    {(stats.average_rating || 0).toFixed(1)} ({stats.total_ratings || 0} ocjena)
                  </span>
                </div>
              )}
            </div>

            <p className={styles.description}>{recipe.description}</p>

            <div className={styles.infoGrid}>
              <div className={styles.infoItem}>
                <span className={styles.infoIcon}>⏱️</span>
                <span className={styles.infoLabel}>Priprema</span>
                <span className={styles.infoValue}>{recipe.prep_time_min} min</span>
              </div>
              <div className={styles.infoItem}>
                <span className={styles.infoIcon}>🔥</span>
                <span className={styles.infoLabel}>Kuvanje</span>
                <span className={styles.infoValue}>{recipe.cook_time_min} min</span>
              </div>
              <div className={styles.infoItem}>
                <span className={styles.infoIcon}>🍽️</span>
                <span className={styles.infoLabel}>Porcije</span>
                <span className={styles.infoValue}>{recipe.servings}</span>
              </div>
              <div className={styles.infoItem}>
                <span className={styles.infoIcon}>⏳</span>
                <span className={styles.infoLabel}>Ukupno</span>
                <span className={styles.infoValue}>{recipe.prep_time_min + recipe.cook_time_min} min</span>
              </div>
            </div>

            {/* SASTOJCI */}
            <div className={styles.section}>
              <h2>🥗 Sastojci</h2>
              {ingredients.length > 0 ? (
                <div className={styles.ingredientsList}>
                  {ingredients.map((ing, i) => (
                    <div key={i} className={styles.ingredientItem}>
                      <span className={styles.ingredientDot}>•</span>
                      <span className={styles.ingredientName}>{ing.name || ing.ingredient_name}</span>
                      <span className={styles.ingredientQty}>
                        {ing.quantity} {ing.unit}
                      </span>
                    </div>
                  ))}
                </div>
              ) : (
                <p className={styles.noIngredients}>Nema podataka o sastojcima</p>
              )}
            </div>

            {/* UPUTE */}
            <div className={styles.section}>
              <h2>📋 Upute za pripremu</h2>
              <div className={styles.instructions}>
                {recipe.instructions.split('\n').map((step, i) => (
                  step.trim() && <p key={i} className={styles.step}>{step}</p>
                ))}
              </div>
            </div>
          </>
        )}
      </div>

      {/* OCJENE */}
      <div className={styles.card}>
        <h2>⭐ Ocjene</h2>

        {user ? (
          <form onSubmit={handleRate} className={styles.rateForm}>
            <div className={styles.rateStars}>
              <span>Vaša ocjena:</span>
              <StarRating rating={score} onRate={setScore} size={28} />
              {score > 0 && <span className={styles.scoreLabel}>{score}/5</span>}
            </div>
            <input
              type="text"
              placeholder="Dodajte komentar (opcionalno)..."
              value={comment}
              onChange={(e) => setComment(e.target.value)}
              className={styles.commentInput}
            />
            <button type="submit" className={styles.rateBtn}>Ocijeni</button>
          </form>
        ) : (
          <p className={styles.loginPrompt}>
            <a href="/login">Prijavite se</a> da biste ocijenili recept.
          </p>
        )}

        <div className={styles.ratingsList}>
          {ratings.length === 0 ? (
            <p className={styles.noRatings}>Još nema ocjena. Budite prvi!</p>
          ) : (
            ratings.map((r, i) => (
              <div key={r.id || i} className={styles.ratingItem}>
                <div className={styles.ratingHeader}>
                  <strong>{r.username}</strong>
                  <StarRating rating={r.score} readonly size={16} />
                </div>
                {r.comment && <p className={styles.ratingComment}>{r.comment}</p>}
              </div>
            ))
          )}
        </div>
      </div>
    </div>
  );
}

export default RecipeDetail;