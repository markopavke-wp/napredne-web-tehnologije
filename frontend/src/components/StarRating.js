import { useState } from 'react';
import styles from './StarRating.module.css';

function StarRating({ rating = 0, onRate, readonly = false, size = 24 }) {
  const [hover, setHover] = useState(0);

  return (
    <div className={styles.stars}>
      {[1, 2, 3, 4, 5].map((star) => (
        <span
          key={star}
          className={`${styles.star} ${star <= (hover || rating) ? styles.filled : ''} ${readonly ? styles.readonly : ''}`}
          style={{ fontSize: size }}
          onClick={() => !readonly && onRate && onRate(star)}
          onMouseEnter={() => !readonly && setHover(star)}
          onMouseLeave={() => !readonly && setHover(0)}
        >
          ★
        </span>
      ))}
    </div>
  );
}

export default StarRating;