import React, { useContext, useState } from 'react';
import { ThemeContext } from '../App';
import { 
  TrashIcon, 
  InformationCircleIcon, 
  ArrowDownTrayIcon, 
  StarIcon
} from '@heroicons/react/24/outline';
import { StarIcon as StarIconSolid } from '@heroicons/react/24/solid';

const GalleryCard = ({ 
  item, 
  onView, 
  onDelete, 
  onToggleFavorite, 
  onShowInfo, 
  onSelect,
  selected
}) => {
  const { theme } = useContext(ThemeContext);
  const [isHovered, setIsHovered] = useState(false);
  
  // For animated WebP files with conversion in progress
  const [conversionProgress, setConversionProgress] = useState(0);
  const [conversionStatus, setConversionStatus] = useState('');
  
  const handleDownload = (e) => {
    e.stopPropagation();
    
    // Create a temporary link element to download the file
    const link = document.createElement('a');
    link.href = `/api/static/images/output/${item.source}`;
    link.download = ''; // Use server filename
    link.style.display = 'none';
    
    // Add to DOM, click, then remove
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
  };
  
  return (
    <div className="flex justify-center">
      <div 
        className={`relative w-full aspect-square rounded-lg overflow-hidden shadow-md transition-all duration-300 hover:-translate-y-1 hover:scale-105 hover:shadow-lg group ${
          theme === 'dark' ? 'bg-card-dark' : 'bg-card-light'
        }`}
        onMouseEnter={() => setIsHovered(true)}
        onMouseLeave={() => setIsHovered(false)}
      >
        {/* Checkbox for selecting multiple items */}
        <input 
          type="checkbox" 
          className="file-checkbox absolute top-2 left-2 z-10 scale-125 cursor-pointer opacity-70 hover:opacity-100 checked:opacity-100 transition-opacity" 
          data-filename={item.filename}
          checked={selected}
          onChange={() => onSelect(item.filename)}
        />
        
        {/* Action buttons */}
        <div className="absolute top-0 right-0 z-10 flex flex-col gap-2 p-2">
          <button 
            className="trash-icon w-8 h-8 rounded-full bg-black/50 text-white flex items-center justify-center opacity-0 translate-x-2 transition-all hover:bg-red-600/90 hover:opacity-100 group-hover:opacity-80 group-hover:translate-x-0" 
            data-filename={item.filename} 
            onClick={(e) => {
              e.stopPropagation();
              onDelete(item.filename);
            }}
            aria-label="Delete file"
          >
            <TrashIcon className="w-4 h-4" />
          </button>
          
          <button 
            className="info-icon w-8 h-8 rounded-full bg-black/50 text-white flex items-center justify-center opacity-0 translate-x-2 transition-all hover:bg-blue-600/90 hover:opacity-100 group-hover:opacity-80 group-hover:translate-x-0" 
            onClick={(e) => {
              e.stopPropagation();
              onShowInfo(item);
            }}
            aria-label="Show file information"
          >
            <InformationCircleIcon className="w-4 h-4" />
          </button>
          
          <button 
            className="download-icon w-8 h-8 rounded-full bg-black/50 text-white flex items-center justify-center opacity-0 translate-x-2 transition-all hover:bg-green-600/90 hover:opacity-100 group-hover:opacity-80 group-hover:translate-x-0" 
            onClick={handleDownload}
            aria-label="Download file"
          >
            <ArrowDownTrayIcon className="w-4 h-4" />
          </button>

          <button 
            className="favorite-icon w-8 h-8 rounded-full bg-black/50 text-white flex items-center justify-center opacity-0 translate-x-2 transition-all hover:bg-yellow-500/90 hover:opacity-100 group-hover:opacity-80 group-hover:translate-x-0" 
            onClick={(e) => {
              e.stopPropagation();
              onToggleFavorite(item.filename, !item.is_favorite);
            }}
            aria-label={item.is_favorite ? "Remove from favorites" : "Add to favorites"}
          >
            {item.is_favorite ? (
              <StarIconSolid className="w-4 h-4 text-yellow-400" />
            ) : (
              <StarIcon className="w-4 h-4" />
            )}
          </button>
        </div>
        
        {/* Thumbnail */}
        <div 
          className="w-full h-full cursor-pointer"
          onClick={() => onView(item)}
        >
          {/* Thumbnail image */}
          <img 
            src={`/api/serve-thumbnail/${item.thumbnail}`}
            alt={item.filename}
            loading="lazy"
            className="w-full h-full object-cover transition-transform duration-500 group-hover:scale-105"
          />
          
          {/* Video indicator for video files */}
          {item.is_video && (
            <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 text-4xl z-10 opacity-70 pointer-events-none">
              {item.is_converted_webp ? (
                <span className="text-2xl">🔄</span>
              ) : item.is_webm ? (
                <span className="text-2xl">🎬</span>
              ) : (
                <div className="bg-black/50 rounded-full p-2">
                  <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" className="w-8 h-8 text-white">
                    <path fillRule="evenodd" d="M4.5 5.653c0-1.426 1.529-2.33 2.779-1.643l11.54 6.348c1.295.712 1.295 2.573 0 3.285L7.28 19.991c-1.25.687-2.779-.217-2.779-1.643V5.653z" clipRule="evenodd" />
                  </svg>
                </div>
              )}
            </div>
          )}
          
          {/* Loading shimmer effect */}
          <div className="absolute top-0 left-0 w-full h-full bg-gradient-to-r from-white/5 via-white/15 to-white/5 dark:from-white/5 dark:via-white/15 dark:to-white/5 bg-200% animate-shimmer pointer-events-none"></div>
          
          {/* Conversion progress for WebP files */}
          {item.filename.toLowerCase().endsWith('.webp') && (
            <div className="absolute bottom-0 left-0 w-full bg-black/70 p-1 z-20">
              <div className="w-full h-2.5 bg-white/20 rounded-sm overflow-hidden mb-1">
                <div className="progress-fill h-full bg-primary-dark" style={{ width: `${conversionProgress}%` }}></div>
              </div>
              <div className="text-white text-xs text-center whitespace-nowrap overflow-hidden text-ellipsis">
                {conversionStatus || 'Checking...'}
              </div>
            </div>
          )}
          
          {/* Filename display on hover */}
          <div className={`absolute bottom-0 left-0 w-full bg-black/50 text-white py-1 px-2 text-xs text-center whitespace-nowrap 
                      overflow-hidden text-ellipsis transition-all duration-300 ${
                        isHovered ? 'opacity-100 translate-y-0' : 'opacity-0 translate-y-full'
                      }`}>
            {item.filename.length > 30 ? `${item.filename.substring(0, 27)}...` : item.filename}
          </div>
        </div>
      </div>
    </div>
  );
};

export default GalleryCard;