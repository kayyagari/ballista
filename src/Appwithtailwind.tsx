import React from 'react'

const Appwithtailwind = () => {
  return (
    <div className='bg-gray-400 h-screen w-screen'>
      <button className='cursor-pointer' onClick={()=> window.location.href = '/'}>go back </button>
    </div>
  )
}

export default Appwithtailwind
