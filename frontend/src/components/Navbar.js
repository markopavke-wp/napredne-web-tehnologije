import { Link, useLocation } from 'react-router-dom';
import { useAuth } from '../context/AuthContext';
import { useTheme } from '../context/ThemeContext';
import styles from './Navbar.module.css';

function Navbar() {
  const { user, logout } = useAuth();
  const { theme, toggleTheme } = useTheme();
  const location = useLocation();

  const isActive = (path) => location.pathname === path ? styles.active : '';

  return (
    <nav className={styles.navbar}>
      <Link to="/" className={styles.brand}>
        <span className={styles.logo}>🍽️</span>
        <span className={styles.brandText}>RecipeApp</span>
      </Link>

      <div className={styles.links}>
        <Link to="/" className={`${styles.link} ${isActive('/')}`}>Recepti</Link>
        <Link to="/search" className={`${styles.link} ${isActive('/search')}`}>Pretraga</Link>

        {user ? (
          <>
            <Link to="/create" className={`${styles.link} ${isActive('/create')}`}>Novi Recept</Link>
            <Link to="/fridge" className={`${styles.link} ${isActive('/fridge')}`}>Frižider</Link>
            <button onClick={logout} className={styles.logoutBtn}>Odjavi se</button>
          </>
        ) : (
          <>
            <Link to="/login" className={`${styles.link} ${isActive('/login')}`}>Prijava</Link>
            <Link to="/register" className={`${styles.link} ${isActive('/register')}`}>Registracija</Link>
          </>
        )}

        <button onClick={toggleTheme} className={styles.themeBtn}>
          {theme === 'light' ? '🌙' : '☀️'}
        </button>
      </div>
    </nav>
  );
}

export default Navbar;