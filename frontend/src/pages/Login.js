import { useState } from 'react';
import { useNavigate, Link } from 'react-router-dom';
import { useAuth } from '../context/AuthContext';
import api from '../api/axios';
import Toast from '../components/Toast';
import styles from './Login.module.css';

function Login() {
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [toast, setToast] = useState(null);
  const [loading, setLoading] = useState(false);
  const { login } = useAuth();
  const navigate = useNavigate();

  const handleSubmit = async (e) => {
    e.preventDefault();
    setToast(null);
    setLoading(true);
    try {
      const res = await api.post('/auth/login', { email, password });
      login(res.data.token);
      setToast({ message: 'Uspješna prijava! Preusmjeravanje...', type: 'success' });
      setTimeout(() => navigate('/'), 1000);
    } catch (err) {
      setToast({ message: err.response?.data?.error || 'Pogrešan email ili lozinka', type: 'error' });
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className={styles.page}>
      {toast && <Toast message={toast.message} type={toast.type} onClose={() => setToast(null)} />}
      <div className={styles.card}>
        <div className={styles.header}>
          <span className={styles.icon}>👤</span>
          <h2>Prijava</h2>
          <p className={styles.subtitle}>Dobro došli nazad!</p>
        </div>
        <form onSubmit={handleSubmit} className={styles.form}>
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
              placeholder="••••••••"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              required
            />
          </div>
          <button type="submit" className={styles.submit} disabled={loading}>
            {loading ? 'Prijavljivanje...' : 'Prijavi se'}
          </button>
        </form>
        <p className={styles.footer}>
          Nemate nalog? <Link to="/register">Registrujte se</Link>
        </p>
      </div>
    </div>
  );
}

export default Login;