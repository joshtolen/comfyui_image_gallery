import React, { useContext, Fragment } from 'react';
import { Dialog, Transition } from '@headlessui/react';
import { ThemeContext } from '../App';
import { XMarkIcon } from '@heroicons/react/24/outline';

const InfoModal = ({ isOpen, onClose, fileInfo }) => {
  const { theme } = useContext(ThemeContext);
  
  if (!fileInfo) {
    return null;
  }
  
  return (
    <Transition appear show={isOpen} as={Fragment}>
      <Dialog as="div" className="relative z-50" onClose={onClose}>
        <Transition.Child
          as={Fragment}
          enter="ease-out duration-300"
          enterFrom="opacity-0"
          enterTo="opacity-100"
          leave="ease-in duration-200"
          leaveFrom="opacity-100"
          leaveTo="opacity-0"
        >
          <div className="fixed inset-0 bg-black/75" />
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
              <Dialog.Panel 
                className="w-full max-w-md transform overflow-hidden rounded-lg text-left align-middle shadow-xl transition-all"
                style={{ 
                  backgroundColor: theme === 'dark' ? '#2d2d2d' : '#ffffff',
                  color: theme === 'dark' ? 'white' : '#1f2937'
                }}
              >
                <div className={`p-4 border-b ${
                  theme === 'dark' ? 'border-white/10' : 'border-black/10'
                }`}>
                  <div className="flex items-center justify-between">
                    <Dialog.Title as="h3" className="text-lg font-semibold">
                      File Information
                    </Dialog.Title>
                    <button
                      type="button"
                      className="rounded-full p-1 opacity-70 hover:opacity-100 hover:bg-black/10 transition-all"
                      onClick={onClose}
                    >
                      <XMarkIcon className="h-6 w-6" aria-hidden="true" />
                    </button>
                  </div>
                </div>
                
                <div className="p-6">
                  <div className="flex mb-4">
                    <span className="flex-1 font-bold mr-2">Filename:</span>
                    <span className="flex-2 break-all">{fileInfo.filename}</span>
                  </div>
                  <div className="flex mb-4">
                    <span className="flex-1 font-bold mr-2">Type:</span>
                    <span className="flex-2">{fileInfo.type}</span>
                  </div>
                  <div className="flex mb-4">
                    <span className="flex-1 font-bold mr-2">Size:</span>
                    <span className="flex-2">{fileInfo.size}</span>
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

export default InfoModal;