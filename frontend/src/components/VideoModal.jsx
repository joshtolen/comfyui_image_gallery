import React, { useContext, Fragment, useRef, useEffect } from 'react';
import { Dialog, Transition } from '@headlessui/react';
import { ThemeContext } from '../App';
import { XMarkIcon } from '@heroicons/react/24/outline';

const VideoModal = ({ isOpen, onClose, videoFile }) => {
  const { theme } = useContext(ThemeContext);
  const videoRef = useRef(null);
  
  useEffect(() => {
    // Pause the video when closing the modal
    if (!isOpen && videoRef.current) {
      videoRef.current.pause();
    }
  }, [isOpen]);
  
  // Determine the MIME type based on the file extension
  const getMimeType = (filename) => {
    if (!filename) return 'video/mp4';
    
    if (filename.toLowerCase().endsWith('.webm')) {
      return 'video/webm';
    } else if (filename.toLowerCase().endsWith('.mkv')) {
      return 'video/x-matroska';
    } else if (filename.toLowerCase().endsWith('.avi')) {
      return 'video/x-msvideo';
    } else if (filename.toLowerCase().endsWith('.mov')) {
      return 'video/quicktime';
    }
    
    return 'video/mp4';
  };
  
  // If no video file is provided, don't render
  if (!videoFile) {
    return null;
  }
  
  const videoSrc = `/api/static/images/output/${videoFile.source}`;
  const mimeType = getMimeType(videoFile.source);
  
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
          <div className="flex min-h-full items-center justify-center p-4 text-center">
            <Transition.Child
              as={Fragment}
              enter="ease-out duration-300"
              enterFrom="opacity-0 scale-95"
              enterTo="opacity-100 scale-100"
              leave="ease-in duration-200"
              leaveFrom="opacity-100 scale-100"
              leaveTo="opacity-0 scale-95"
            >
              <Dialog.Panel className="w-full max-w-4xl transform overflow-hidden rounded-lg bg-black text-left align-middle shadow-xl transition-all">
                <div className="relative">
                  <div className="p-4 border-b border-white/10 bg-black/70">
                    <div className="flex items-center justify-between">
                      <Dialog.Title as="h3" className="text-lg font-semibold text-white">
                        {videoFile.filename}
                      </Dialog.Title>
                      <button
                        type="button"
                        className="rounded-full p-1 text-white opacity-70 hover:opacity-100 hover:bg-white/10 transition-all"
                        onClick={onClose}
                      >
                        <XMarkIcon className="h-6 w-6" aria-hidden="true" />
                      </button>
                    </div>
                  </div>
                  
                  <div className="w-full">
                    <video 
                      ref={videoRef}
                      className="w-full max-h-[70vh]" 
                      controls 
                      autoPlay 
                      playsInline
                    >
                      <source src={videoSrc} type={mimeType} />
                      Your browser doesn't support HTML5 video playback.
                    </video>
                  </div>
                </div>
              </Dialog.Panel>
            </Transition.Child>
          </div>
        </div>
      </Dialog>
    </Transition>
  );
};

export default VideoModal;