import React, { useContext } from 'react';
import { ThemeContext } from '../App';
import { Link } from 'react-router-dom';

const Pagination = ({ currentPage, totalPages, fileType = 'all' }) => {
  const { theme } = useContext(ThemeContext);
  
  // If there's only one page, don't show pagination
  if (totalPages <= 1) {
    return null;
  }
  
  // Calculate the range of page numbers to display
  const getPageRange = () => {
    let start = Math.max(1, currentPage - 2);
    let end = Math.min(totalPages, start + 4);
    
    // Adjust start if end is maxed out
    if (end === totalPages) {
      start = Math.max(1, end - 4);
    }
    
    return Array.from({ length: end - start + 1 }, (_, i) => start + i);
  };
  
  const pageRange = getPageRange();
  
  const getPageUrl = (page) => {
    return `/type/${fileType}?page=${page}`;
  };
  
  return (
    <div className="flex justify-center my-6 pb-8">
      <div className="flex flex-wrap justify-center gap-2 max-w-full px-4">
        {currentPage > 1 && (
          <>
            <Link
              to={getPageUrl(1)}
              className={`px-3 py-2 rounded-md transition-all hover:-translate-y-0.5 ${
                theme === 'dark' ? 'bg-card-dark text-white' : 'bg-card-light text-gray-800 shadow-sm'
              }`}
              aria-label="First page"
            >
              &laquo; First
            </Link>
            <Link
              to={getPageUrl(currentPage - 1)}
              className={`px-3 py-2 rounded-md transition-all hover:-translate-y-0.5 ${
                theme === 'dark' ? 'bg-card-dark text-white' : 'bg-card-light text-gray-800 shadow-sm'
              }`}
              aria-label="Previous page"
            >
              Previous
            </Link>
          </>
        )}
        
        <div className="flex flex-wrap gap-2 justify-center">
          {pageRange.map(page => (
            <Link
              key={page}
              to={getPageUrl(page)}
              className={`px-3 py-2 rounded-md font-medium transition-all hover:-translate-y-0.5 ${
                page === currentPage
                  ? (theme === 'dark' 
                    ? 'bg-primary-dark text-white font-bold' 
                    : 'bg-primary-light text-white font-bold')
                  : (theme === 'dark' 
                    ? 'bg-card-dark text-white' 
                    : 'bg-card-light text-gray-800 shadow-sm')
              }`}
              aria-label={`Page ${page}`}
              aria-current={page === currentPage ? 'page' : undefined}
            >
              {page}
            </Link>
          ))}
        </div>
        
        {currentPage < totalPages && (
          <>
            <Link
              to={getPageUrl(currentPage + 1)}
              className={`px-3 py-2 rounded-md transition-all hover:-translate-y-0.5 ${
                theme === 'dark' ? 'bg-card-dark text-white' : 'bg-card-light text-gray-800 shadow-sm'
              }`}
              aria-label="Next page"
            >
              Next
            </Link>
            <Link
              to={getPageUrl(totalPages)}
              className={`px-3 py-2 rounded-md transition-all hover:-translate-y-0.5 ${
                theme === 'dark' ? 'bg-card-dark text-white' : 'bg-card-light text-gray-800 shadow-sm'
              }`}
              aria-label="Last page"
            >
              Last &raquo;
            </Link>
          </>
        )}
      </div>
    </div>
  );
};

export default Pagination;