import { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import api from '../api/axios';
import Toast from '../components/Toast';
import styles from './CreateRecipe.module.css';

function CreateRecipe() {
  const [title, setTitle] = useState('');
  const [description, setDescription] = useState('');
  const [instructions, setInstructions] = useState('');
  const [prepTime, setPrepTime] = useState(10);
  const [cookTime, setCookTime] = useState(15);
  const [servings, setServings] = useState(4);
  const [allIngredients, setAllIngredients] = useState([]);
  const [selected, setSelected] = useState([]);
  const [toast, setToast] = useState(null);
  const [loading, setLoading] = useState(false);
  const navigate = useNavigate();

  useEffect(() => {
    api.get('/ingredients').then((res) => setAllIngredients(res.data)).catch(console.error);
  }, []);

  const addIngredient = () => {
    setSelected([...selected, { ingredient_id: '', quantity: 100, unit: 'g' }]);
  };

  const updateIngredient = (index, field, value) => {
    const updated = [...selected];
    updated[index][field] = field === 'quantity' ? Number(value) : value;
    setSelected(updated);
  };

  const removeIngredient = (index) => {
    setSelected(selected.filter((_, i) => i !== index));
  };

  const handleSubmit = async (e) => {
    e.preventDefault();
    if (selected.length === 0) {
      setToast({ message: 'Dodajte bar jedan sastojak', type: 'error' });
      return;
    }
    const invalid = selected.some((s) => !s.ingredient_id);
    if (invalid) {
      setToast({ message: 'Izaberite sve sastojke', type: 'error' });
      return;
    }
    setLoading(true);
    try {
      const res = await api.post('/recipes', {
        title, description, instructions,
        prep_time_min: Number(prepTime),
        cook_time_min: Number(cookTime),
        servings: Number(servings),
        ingredients: selected.map((s) => ({
          ingredient_id: s.ingredient_id,
          quantity: s.quantity,
          unit: s.unit,
        })),
      });
      setToast({ message: 'Recept uspješno kreiran!', type: 'success' });
      setTimeout(() => navigate(`/recipes/${res.data.id}`), 1200);
    } catch (err) {
      setToast({ message: err.response?.data?.error || 'Greška pri kreiranju recepta', type: 'error' });
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className={styles.page}>
      {toast && <Toast message={toast.message} type={toast.type} onClose={() => setToast(null)} />}
      <div className={styles.card}>
        <div className={styles.header}>
          <span className={styles.icon}>👨‍🍳</span>
          <h2>Novi Recept</h2>
        </div>

        <form onSubmit={handleSubmit} className={styles.form}>
          <div className={styles.field}>
            <label>Naziv recepta</label>
            <input type="text" placeholder="npr. Palačinke" value={title} onChange={(e) => setTitle(e.target.value)} required />
          </div>

          <div className={styles.field}>
            <label>Opis</label>
            <textarea placeholder="Kratak opis recepta..." value={description} onChange={(e) => setDescription(e.target.value)} required rows={2} />
          </div>

          <div className={styles.field}>
            <label>Upute za pripremu</label>
            <textarea placeholder="Korak po korak..." value={instructions} onChange={(e) => setInstructions(e.target.value)} required rows={5} />
          </div>

          <div className={styles.row}>
            <div className={styles.field}>
              <label>⏱️ Priprema (min)</label>
              <input type="number" value={prepTime} onChange={(e) => setPrepTime(e.target.value)} min={0} />
            </div>
            <div className={styles.field}>
              <label>🔥 Kuvanje (min)</label>
              <input type="number" value={cookTime} onChange={(e) => setCookTime(e.target.value)} min={0} />
            </div>
            <div className={styles.field}>
              <label>🍽️ Porcije</label>
              <input type="number" value={servings} onChange={(e) => setServings(e.target.value)} min={1} />
            </div>
          </div>

          <div className={styles.ingredientsSection}>
            <div className={styles.ingredientsHeader}>
              <h3>🥗 Sastojci</h3>
              <button type="button" onClick={addIngredient} className={styles.addBtn}>+ Dodaj</button>
            </div>

            {selected.length === 0 && (
              <p className={styles.emptyMsg}>Kliknite "Dodaj" za dodavanje sastojaka</p>
            )}

            {selected.map((ing, i) => (
              <div key={i} className={styles.ingredientRow}>
                <select value={ing.ingredient_id} onChange={(e) => updateIngredient(i, 'ingredient_id', e.target.value)} required>
                  <option value="">Izaberi...</option>
                  {allIngredients.map((a) => (
                    <option key={a.id} value={a.id}>{a.name}</option>
                  ))}
                </select>
                <input type="number" placeholder="Količina" value={ing.quantity} onChange={(e) => updateIngredient(i, 'quantity', e.target.value)} min={1} />
                <select value={ing.unit} onChange={(e) => updateIngredient(i, 'unit', e.target.value)}>
                  <option value="g">g</option>
                  <option value="ml">ml</option>
                  <option value="kom">kom</option>
                  <option value="kasika">kašika</option>
                  <option value="kasicica">kašičica</option>
                </select>
                <button type="button" onClick={() => removeIngredient(i)} className={styles.removeBtn}>✕</button>
              </div>
            ))}
          </div>

          <button type="submit" className={styles.submit} disabled={loading}>
            {loading ? 'Kreiranje...' : '✨ Kreiraj recept'}
          </button>
        </form>
      </div>
    </div>
  );
}

export default CreateRecipe;