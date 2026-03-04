import { Link } from 'react-router-dom';
import StarRating from './StarRating';
import styles from './RecipeCard.module.css';

function RecipeCard({ recipe, stats }) {
  const totalTime = recipe.prep_time_min + recipe.cook_time_min;

  return (
    <Link to={`/recipes/${recipe.id}`} className={styles.card}>
      <div className={styles.header}>
        <h3 className={styles.title}>{recipe.title}</h3>
        {stats && (
          <div className={styles.rating}>
            <StarRating rating={Math.round(stats.average_rating)} readonly size={14} />
            <span className={styles.ratingCount}>({stats.total_ratings})</span>
          </div>
        )}
      </div>
      <p className={styles.description}>{recipe.description}</p>
      <div className={styles.meta}>
        <span className={styles.metaItem}>👤 {recipe.username}</span>
        <span className={styles.metaItem}>⏱️ {totalTime} min</span>
        <span className={styles.metaItem}>🍽️ {recipe.servings} porcija</span>
      </div>
    </Link>
  );
}

export default RecipeCard;