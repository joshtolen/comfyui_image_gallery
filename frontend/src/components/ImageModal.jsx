import React, { Fragment, useState } from 'react';
import { Dialog, Transition } from '@headlessui/react';
import { 
  XMarkIcon, 
  ArrowsPointingOutIcon, 
  ArrowsPointingInIcon,
  ChevronLeftIcon,
  ChevronRightIcon
} from '@heroicons/react/24/outline';

const ImageModal = ({ isOpen, onClose, imageFile, onNext, onPrevious, hasNext, hasPrevious }) => {
  const [isZoomed, setIsZoomed] = useState(false);
  
  // If no image file is provided, don't render
  if (!imageFile) {
    return null;
  }
  
  const imageSrc = `/api/static/images/output/${imageFile.source}`;
  
  const toggleZoom = () => {
    setIsZoomed(!isZoomed);
  };
  
  return (
    <Transition appear show={isOpen} as={Fragment}>
      <Dialog 
        as="div" 
        className="relative z-50" 
        onClose={onClose}
      >
        <Transition.Child
          as={Fragment}
          enter="ease-out duration-300"
          enterFrom="opacity-0"
          enterTo="opacity-100"
          leave="ease-in duration-200"
          leaveFrom="opacity-100"
          leaveTo="opacity-0"
        >
          <div className="fixed inset-0 bg-black/90" />
        </Transition.Child>

        <div className="fixed inset-0 overflow-y-auto">
          <div className="flex min-h-full items-center justify-center text-center">
            <Transition.Child
              as={Fragment}
              enter="ease-out duration-300"
              enterFrom="opacity-0 scale-95"
              enterTo="opacity-100 scale-100"
              leave="ease-in duration-200"
              leaveFrom="opacity-100 scale-100"
              leaveTo="opacity-0 scale-95"
            >
              <Dialog.Panel className="w-full h-full transform text-left align-middle transition-all">
                <div className="absolute top-4 right-4 flex gap-4 z-10">
                  <button
                    type="button"
                    className="rounded-full p-2 bg-black/50 text-white opacity-70 hover:opacity-100 hover:bg-black/70 transition-all"
                    onClick={toggleZoom}
                  >
                    {isZoomed ? (
                      <ArrowsPointingInIcon className="h-6 w-6" aria-hidden="true" />
                    ) : (
                      <ArrowsPointingOutIcon className="h-6 w-6" aria-hidden="true" />
                    )}
                  </button>
                  <button
                    type="button"
                    className="rounded-full p-2 bg-black/50 text-white opacity-70 hover:opacity-100 hover:bg-black/70 transition-all"
                    onClick={onClose}
                  >
                    <XMarkIcon className="h-6 w-6" aria-hidden="true" />
                  </button>
                </div>
                
                {/* Image navigation arrows */}
                {hasPrevious && (
                  <button
                    type="button"
                    className="absolute left-4 top-1/2 transform -translate-y-1/2 rounded-full p-2 bg-black/50 text-white opacity-70 hover:opacity-100 hover:bg-black/70 transition-all"
                    onClick={onPrevious}
                  >
                    <ChevronLeftIcon className="h-8 w-8" aria-hidden="true" />
                  </button>
                )}
                
                {hasNext && (
                  <button
                    type="button"
                    className="absolute right-4 top-1/2 transform -translate-y-1/2 rounded-full p-2 bg-black/50 text-white opacity-70 hover:opacity-100 hover:bg-black/70 transition-all"
                    onClick={onNext}
                  >
                    <ChevronRightIcon className="h-8 w-8" aria-hidden="true" />
                  </button>
                )}
                
                {/* Image container */}
                <div 
                  className={`w-full h-full flex items-center justify-center overflow-auto ${
                    isZoomed ? 'cursor-zoom-out' : 'cursor-zoom-in'
                  }`} 
                  onClick={toggleZoom}
                >
                  <img 
                    src={imageSrc} 
                    alt={imageFile.filename} 
                    className={`max-h-screen transition-transform duration-300 ${
                      isZoomed ? 'max-w-none transform scale-150' : 'max-w-full'
                    }`}
                  />
                </div>
                
                {/* Filename display at bottom */}
                <div className="absolute bottom-0 left-0 w-full bg-black/50 text-white py-2 px-4 text-center">
                  {imageFile.filename}
                </div>
              </Dialog.Panel>
            </Transition.Child>
          </div>
        </div>
      </Dialog>
    </Transition>
  );
};

export default ImageModal;