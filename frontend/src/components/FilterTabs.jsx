import React, { useContext } from 'react';
import { Link, useParams } from 'react-router-dom';
import { ThemeContext } from '../App';

const FilterTabs = ({ stats }) => {
  const { theme } = useContext(ThemeContext);
  const { fileType } = useParams();
  
  // Default to 'all' if no type is specified
  const currentType = fileType || 'all';
  
  // Define our tab options
  const tabs = [
    { id: 'all', label: 'All', count: stats.total_count },
    { id: 'images', label: 'Images', count: stats.image_count },
    { id: 'videos', label: 'Videos', count: stats.video_count },
    { id: 'favorites', label: 'Favorites⭐', count: stats.favorite_count },
  ];
  
  return (
    <div className="flex justify-center items-center gap-4 flex-wrap my-4 px-4">
      {tabs.map(tab => (
        <Link
          key={tab.id}
          to={`/type/${tab.id}`}
          className={`px-4 py-2 rounded-md font-medium transition-all hover:-translate-y-0.5 ${
            currentType === tab.id 
              ? (theme === 'dark' 
                ? 'bg-primary-dark text-white font-bold' 
                : 'bg-primary-light text-white font-bold')
              : (theme === 'dark' 
                ? 'bg-card-dark text-white' 
                : 'bg-card-light text-gray-800 shadow-sm')
          }`}
        >
          {tab.label} ({tab.count || 0})
        </Link>
      ))}
    </div>
  );
};

export default FilterTabs;