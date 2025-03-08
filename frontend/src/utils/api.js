import axios from 'axios';

// Create axios instance with common settings
const api = axios.create({
  baseURL: '/api',
  timeout: 30000, // 30 seconds
  headers: {
    'Content-Type': 'application/json',
  },
});

// Gallery data API
export const fetchGalleryData = async (fileType = 'all', page = 1) => {
  try {
    const response = await api.get(`/?type=${fileType}&page=${page}`);
    return response.data;
  } catch (error) {
    console.error('Error fetching gallery data:', error);
    throw error;
  }
};

// File operations
export const deleteFile = async (filename) => {
  try {
    const response = await api.post('/delete-file', { file: filename });
    return response.data;
  } catch (error) {
    console.error('Error deleting file:', error);
    throw error;
  }
};

export const deleteFiles = async (files) => {
  try {
    const response = await api.post('/delete-files', { files });
    return response.data;
  } catch (error) {
    console.error('Error deleting files:', error);
    throw error;
  }
};

// Favorites
export const toggleFavorite = async (file, is_favorite) => {
  try {
    const response = await api.post('/toggle-favorite', { file, is_favorite });
    return response.data;
  } catch (error) {
    console.error('Error toggling favorite status:', error);
    throw error;
  }
};

// File info
export const getFileInfo = async (filename) => {
  try {
    const response = await api.get(`/file-info/${filename}`);
    return response.data;
  } catch (error) {
    console.error('Error getting file info:', error);
    throw error;
  }
};

// WebP conversion progress
export const checkConversionProgress = async (filename) => {
  try {
    const response = await api.get(`/conversion-progress/${filename}`);
    return response.data;
  } catch (error) {
    console.error('Error checking conversion progress:', error);
    throw error;
  }
};

export default api;