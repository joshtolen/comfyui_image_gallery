import React, { useState, useEffect, useCallback } from 'react';
import { useParams, useSearchParams } from 'react-router-dom';
import FilterTabs from '../components/FilterTabs';
import GalleryCard from '../components/GalleryCard';
import Pagination from '../components/Pagination';
import Loading from '../components/Loading';
import InfoModal from '../components/InfoModal';
import VideoModal from '../components/VideoModal';
import ImageModal from '../components/ImageModal';
import { fetchGalleryData, deleteFile, deleteFiles, toggleFavorite } from '../utils/api';

const Gallery = () => {
  // URL params
  const { fileType } = useParams();
  const [searchParams] = useSearchParams();
  const pageParam = searchParams.get('page');
  const currentPage = pageParam ? parseInt(pageParam, 10) : 1;
  
  // State
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [galleryData, setGalleryData] = useState({
    images: [],
    total_pages: 0,
    page: 1,
    file_type: fileType || 'all',
    image_count: 0,
    video_count: 0,
    favorite_count: 0,
    total_count: 0
  });
  
  // Selected items
  const [selectedItems, setSelectedItems] = useState(new Set());
  
  // Modals
  const [infoModalOpen, setInfoModalOpen] = useState(false);
  const [currentInfoFile, setCurrentInfoFile] = useState(null);
  
  const [videoModalOpen, setVideoModalOpen] = useState(false);
  const [currentVideoFile, setCurrentVideoFile] = useState(null);
  
  const [imageModalOpen, setImageModalOpen] = useState(false);
  const [currentImageFile, setCurrentImageFile] = useState(null);
  const [currentImageIndex, setCurrentImageIndex] = useState(-1);
  
  // Load data
  const loadGalleryData = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      
      const type = fileType || 'all';
      const data = await fetchGalleryData(type, currentPage);
      
      setGalleryData({
        images: data.images || [],
        total_pages: data.total_pages || 0,
        page: data.page || 1,
        file_type: type,
        image_count: data.image_count || 0,
        video_count: data.video_count || 0,
        favorite_count: data.favorite_count || 0,
        total_count: data.total_count || 0
      });
    } catch (err) {
      setError('Failed to load gallery data. Please try again.');
      console.error('Error loading gallery data:', err);
    } finally {
      setLoading(false);
    }
  }, [fileType, currentPage]);
  
  useEffect(() => {
    loadGalleryData();
  }, [loadGalleryData]);
  
  // Handle item selection
  const handleSelectItem = (filename) => {
    setSelectedItems(prev => {
      const newSelected = new Set(prev);
      if (newSelected.has(filename)) {
        newSelected.delete(filename);
      } else {
        newSelected.add(filename);
      }
      return newSelected;
    });
  };
  
  // Handle delete single item
  const handleDeleteItem = async (filename) => {
    if (!window.confirm(`Are you sure you want to delete ${filename}?`)) {
      return;
    }
    
    try {
      await deleteFile(filename);
      // Reload data
      loadGalleryData();
    } catch (err) {
      setError('Failed to delete file. Please try again.');
    }
  };
  
  // Handle delete selected items
  const handleDeleteSelected = async () => {
    if (selectedItems.size === 0) {
      alert('Please select at least one file to delete.');
      return;
    }
    
    if (!window.confirm(`Are you sure you want to delete ${selectedItems.size} selected files?`)) {
      return;
    }
    
    try {
      await deleteFiles(Array.from(selectedItems));
      // Clear selections and reload data
      setSelectedItems(new Set());
      loadGalleryData();
    } catch (err) {
      setError('Failed to delete selected files. Please try again.');
    }
  };
  
  // Toggle favorite
  const handleToggleFavorite = async (filename, isFavorite) => {
    try {
      await toggleFavorite(filename, isFavorite);
      
      // Update local state to avoid a full reload
      setGalleryData(prev => ({
        ...prev,
        images: prev.images.map(img => 
          img.filename === filename 
            ? { ...img, is_favorite: isFavorite } 
            : img
        )
      }));
    } catch (err) {
      setError('Failed to update favorite status. Please try again.');
    }
  };
  
  // Show info modal
  const handleShowInfo = (item) => {
    setCurrentInfoFile(item);
    setInfoModalOpen(true);
  };
  
  // Show video modal
  const handleShowVideo = (item) => {
    setCurrentVideoFile(item);
    setVideoModalOpen(true);
  };
  
  // Show image modal
  const handleShowImage = (item, index) => {
    const idx = index !== undefined ? index : galleryData.images.findIndex(img => img.filename === item.filename);
    setCurrentImageFile(item);
    setCurrentImageIndex(idx);
    setImageModalOpen(true);
  };
  
  // Handle view media (dispatch to correct modal)
  const handleViewMedia = (item) => {
    if (item.is_video) {
      handleShowVideo(item);
    } else {
      const index = galleryData.images.findIndex(img => img.filename === item.filename);
      handleShowImage(item, index);
    }
  };
  
  // Navigation for image modal
  const handleNextImage = () => {
    if (currentImageIndex < galleryData.images.length - 1) {
      const nextItem = galleryData.images[currentImageIndex + 1];
      // Skip videos when navigating in image modal
      if (!nextItem.is_video) {
        setCurrentImageFile(nextItem);
        setCurrentImageIndex(currentImageIndex + 1);
      } else {
        // Find next non-video
        let nextIndex = currentImageIndex + 1;
        while (nextIndex < galleryData.images.length) {
          if (!galleryData.images[nextIndex].is_video) {
            setCurrentImageFile(galleryData.images[nextIndex]);
            setCurrentImageIndex(nextIndex);
            break;
          }
          nextIndex++;
        }
      }
    }
  };
  
  const handlePreviousImage = () => {
    if (currentImageIndex > 0) {
      const prevItem = galleryData.images[currentImageIndex - 1];
      // Skip videos when navigating in image modal
      if (!prevItem.is_video) {
        setCurrentImageFile(prevItem);
        setCurrentImageIndex(currentImageIndex - 1);
      } else {
        // Find previous non-video
        let prevIndex = currentImageIndex - 1;
        while (prevIndex >= 0) {
          if (!galleryData.images[prevIndex].is_video) {
            setCurrentImageFile(galleryData.images[prevIndex]);
            setCurrentImageIndex(prevIndex);
            break;
          }
          prevIndex--;
        }
      }
    }
  };
  
  // Check if has next/previous image (for navigation)
  const hasNextImage = () => {
    if (currentImageIndex >= galleryData.images.length - 1) return false;
    
    // Check if there are any non-video items after the current index
    for (let i = currentImageIndex + 1; i < galleryData.images.length; i++) {
      if (!galleryData.images[i].is_video) {
        return true;
      }
    }
    
    return false;
  };
  
  const hasPreviousImage = () => {
    if (currentImageIndex <= 0) return false;
    
    // Check if there are any non-video items before the current index
    for (let i = currentImageIndex - 1; i >= 0; i--) {
      if (!galleryData.images[i].is_video) {
        return true;
      }
    }
    
    return false;
  };
  
  return (
    <div className="container mx-auto px-4">
      {/* Tabs */}
      <FilterTabs stats={galleryData} />
      
      {/* Loading indicator */}
      {loading ? (
        <Loading />
      ) : error ? (
        <div className="text-center text-red-500 my-8 p-4">{error}</div>
      ) : (
        <>
          {/* Gallery grid */}
          <div className="grid grid-cols-auto-fill-280 gap-6 p-6 max-w-[1800px] mx-auto">
            {galleryData.images.map((item) => (
              <GalleryCard 
                key={item.filename}
                item={item}
                onView={handleViewMedia}
                onDelete={handleDeleteItem}
                onToggleFavorite={handleToggleFavorite}
                onShowInfo={handleShowInfo}
                onSelect={handleSelectItem}
                selected={selectedItems.has(item.filename)}
              />
            ))}
          </div>
          
          {/* Empty state */}
          {galleryData.images.length === 0 && (
            <div className="text-center py-12">
              <h3 className="text-xl font-semibold mb-2">No images found</h3>
              <p className="text-gray-500 dark:text-gray-400">
                There are no images or videos matching your current filter.
              </p>
            </div>
          )}
          
          {/* Pagination */}
          <Pagination 
            currentPage={galleryData.page} 
            totalPages={galleryData.total_pages}
            fileType={galleryData.file_type}
          />
        </>
      )}
      
      {/* Modals */}
      <InfoModal 
        isOpen={infoModalOpen} 
        onClose={() => setInfoModalOpen(false)} 
        fileInfo={currentInfoFile} 
      />
      
      <VideoModal 
        isOpen={videoModalOpen} 
        onClose={() => setVideoModalOpen(false)} 
        videoFile={currentVideoFile} 
      />
      
      <ImageModal 
        isOpen={imageModalOpen} 
        onClose={() => setImageModalOpen(false)} 
        imageFile={currentImageFile}
        onNext={handleNextImage}
        onPrevious={handlePreviousImage}
        hasNext={hasNextImage()}
        hasPrevious={hasPreviousImage()}
      />
    </div>
  );
};

export default Gallery;