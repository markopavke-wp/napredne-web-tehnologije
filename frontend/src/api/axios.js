// Axios instanca sa baznim URL-om i automatskim JWT tokenom.
// Svaki API poziv automatski dodaje Authorization header
// ako je korisnik ulogovan.

import axios from 'axios';

const api = axios.create({
  baseURL: '/api',  // Nginx prosljeđuje na API server
});

// Interceptor - dodaje JWT token na svaki zahtjev
api.interceptors.request.use((config) => {
  const token = localStorage.getItem('token');
  if (token) {
    config.headers.Authorization = `Bearer ${token}`;
  }
  return config;
});

export default api;