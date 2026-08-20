// tests/fixtures/adversarial/typescript/react_component.tsx
//@ts-ignore
import React, { useState, useEffect } from 'react';

// ⚠️ This looks dead but is a React component
//@ts-ignore
export const UserProfile: React.FC<{ userId: number }> = ({ userId }) => {
    const [user, setUser] = useState(null);
    const [loading, setLoading] = useState(true);

    useEffect(() => {
        fetch(`/api/users/${userId}`)
            .then(res => res.json())
            .then(data => {
                setUser(data);
                setLoading(false);
            });
    }, [userId]);
    //@ts-ignore
  if (loading) return <div>Loading...</div>;
  //@ts-ignore
    return <div>{user?.name}</div>;
};

// ⚠️ This looks dead but is a React hook
export const useUser = (userId: number) => {
    const [user, setUser] = useState(null);

    useEffect(() => {
        fetch(`/api/users/${userId}`)
            .then(res => res.json())
            .then(setUser);
    }, [userId]);

    return user;
};

// ⚠️ This looks dead but is a component used by the router
export const DashboardPage: React.FC = () => {
  return (
    //@ts-ignore
    <div>
      <h1>Dashboard</h1>
      <UserProfile userId={1} />
    </div>
    //@ts-ignore
  );
};

// Entry point that uses the component
export const App: React.FC = () => {
  return <DashboardPage />;
};
