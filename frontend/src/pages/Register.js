import { useState } from 'react';
import { useNavigate, Link } from 'react-router-dom';
import api from '../api/axios';
import Toast from '../components/Toast';
import styles from './Register.module.css';

function Register() {
  const [username, setUsername] = useState('');
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [toast, setToast] = useState(null);
  const [loading, setLoading] = useState(false);
  const navigate = useNavigate();

  const handleSubmit = async (e) => {
    e.preventDefault();
    setToast(null);
    setLoading(true);
    try {
      await api.post('/auth/register', { username, email, password });
      setToast({ message: 'Registracija uspješna! Preusmjeravanje na prijavu...', type: 'success' });
      setTimeout(() => navigate('/login'), 1500);
    } catch (err) {
      setToast({ message: err.response?.data?.error || 'Greška pri registraciji', type: 'error' });
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className={styles.page}>
      {toast && <Toast message={toast.message} type={toast.type} onClose={() => setToast(null)} />}
      <div className={styles.card}>
        <div className={styles.header}>
          <span className={styles.icon}>✨</span>
          <h2>Registracija</h2>
          <p className={styles.subtitle}>Kreirajte novi nalog</p>
        </div>
        <form onSubmit={handleSubmit} className={styles.form}>
          <div className={styles.field}>
            <label>Korisničko ime</label>
            <input
              type="text"
              placeholder="vaše_ime"
              value={username}
              onChange={(e) => setUsername(e.target.value)}
              required
            />
          </div>
          <div className={styles.field}>
            <label>Email</label>
            <input
              type="email"
              placeholder="vas@email.com"
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              required
            />
          </div>
          <div className={styles.field}>
            <label>Lozinka</label>
            <input
              type="password"
              placeholder="Minimum 6 karaktera"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              required
              minLength={6}
            />
          </div>
          <button type="submit" className={styles.submit} disabled={loading}>
            {loading ? 'Kreiranje naloga...' : 'Registruj se'}
          </button>
        </form>
        <p className={styles.footer}>
          Već imate nalog? <Link to="/login">Prijavite se</Link>
        </p>
      </div>
    </div>
  );
}

export default Register;