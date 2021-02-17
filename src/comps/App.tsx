import React from 'react';
import styles from "./App.module.scss";

function App() {
  return (
    <div className={styles.test}>
      Hello, 
      <div className={styles.hoge}>
        World!
      </div>
    </div>
  );
}

export default App;
