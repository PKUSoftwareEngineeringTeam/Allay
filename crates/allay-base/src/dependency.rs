use std::borrow::Borrow;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use thiserror::Error;

/// A DAG to manage dependencies between points of type `T`
#[derive(Debug, Default)]
pub struct DependencyGraph<T: Hash + Eq + Clone> {
    /// point -> the points that depend on it
    depends_on: HashMap<T, HashSet<T>>,
    /// point -> the points that it depends on
    depended_by: HashMap<T, HashSet<T>>,
}

/// Errors that can occur in dependency graph operations
#[derive(Debug, Error)]
pub enum DependencyError {
    #[error("A point cannot depend on itself")]
    SelfDependency,

    #[error("Adding this dependency would create a circular dependency")]
    CircularDependency,

    #[error("Cannot replace to an existing point")]
    ReplaceToExistingPoint,
}

/// Result type for dependency graph operations
///
/// This is a type alias for [`Result<T, DependencyError>`]
pub type DependencyResult<T> = Result<T, DependencyError>;

impl<T: Hash + Eq + Clone> DependencyGraph<T> {
    fn add_depends_on(&mut self, point: T, dependent: T) {
        self.depends_on.entry(point).or_default().insert(dependent);
    }

    fn add_depended_by(&mut self, point: T, dependency: T) {
        self.depended_by.entry(point).or_default().insert(dependency);
    }

    fn remove_depend_on<Q, R>(&mut self, point: &Q, dependent: &R)
    where
        Q: ?Sized + Eq + Hash,
        R: ?Sized + Eq + Hash,
        T: Borrow<Q> + Borrow<R>,
    {
        if let Some(deps) = self.depends_on.get_mut(point) {
            deps.remove(dependent);
            if deps.is_empty() {
                self.depends_on.remove(point);
            }
        }
    }

    fn remove_depended_by<Q, R>(&mut self, point: &Q, dependency: &R)
    where
        Q: ?Sized + Eq + Hash,
        R: ?Sized + Eq + Hash,
        T: Borrow<Q> + Borrow<R>,
    {
        if let Some(deps) = self.depended_by.get_mut(point) {
            deps.remove(dependency);
            if deps.is_empty() {
                self.depended_by.remove(point);
            }
        }
    }

    /// Get all points that depend on the given point directly
    pub fn depends_on<Q: ?Sized + Eq + Hash>(&self, point: &Q) -> Option<&HashSet<T>>
    where
        T: Borrow<Q>,
    {
        self.depends_on.get(point)
    }

    /// Get all points that the given point directly depends on
    pub fn depended_by<Q: ?Sized + Eq + Hash>(&self, point: &Q) -> Option<&HashSet<T>>
    where
        T: Borrow<Q>,
    {
        self.depended_by.get(point)
    }

    /// Check if `downstream` depends on `upstream` directly
    pub fn is_dependent<Q, R>(&self, downstream: &Q, upstream: &R) -> bool
    where
        Q: ?Sized + Eq + Hash,
        R: ?Sized + Eq + Hash,
        T: Borrow<Q> + Borrow<R>,
    {
        if let Some(deps) = self.depends_on(upstream) {
            deps.contains(downstream)
        } else {
            false
        }
    }

    /// Check if a point exists in the graph
    pub fn exists<Q: ?Sized + Eq + Hash>(&self, point: &Q) -> bool
    where
        T: Borrow<Q>,
    {
        self.depends_on.contains_key(point) || self.depended_by.contains_key(point)
    }

    /// Check if a point is a root (no dependencies)
    /// Returns true if the point does not exist in the graph
    pub fn is_root<Q: ?Sized + Eq + Hash>(&self, point: &Q) -> bool
    where
        T: Borrow<Q>,
    {
        self.depended_by(point).is_none_or(|deps| deps.is_empty())
    }

    /// Check if a point is a leaf (no dependents)
    /// Returns true if the point does not exist in the graph
    pub fn is_leaf<Q: ?Sized + Eq + Hash>(&self, point: &Q) -> bool
    where
        T: Borrow<Q>,
    {
        self.depends_on(point).is_none_or(|deps| deps.is_empty())
    }

    /// Create a new dependency line
    pub fn add_dependency(&mut self, downstream: T, upstream: T) -> DependencyResult<()> {
        if downstream == upstream {
            return Err(DependencyError::SelfDependency);
        }
        if self.is_dependent_recursively(&upstream, &downstream) {
            return Err(DependencyError::CircularDependency);
        }

        self.add_depends_on(upstream.clone(), downstream.clone());
        self.add_depended_by(downstream, upstream);
        Ok(())
    }

    /// Remove a dependency line
    pub fn remove_dependency<Q, R>(&mut self, downstream: &Q, upstream: &R)
    where
        Q: ?Sized + Eq + Hash + PartialEq<R>,
        R: ?Sized + Eq + Hash,
        T: Borrow<Q> + Borrow<R>,
    {
        if downstream == upstream {
            return;
        }

        self.remove_depend_on(upstream, downstream);
        self.remove_depended_by(downstream, upstream);
    }

    /// Remove all dependencies related to a point
    pub fn remove_point<Q: ?Sized + Eq + Hash>(&mut self, point: &Q)
    where
        T: Borrow<Q> + Borrow<T>,
    {
        if let Some(down) = self.depends_on.remove(point) {
            for dep in down {
                self.remove_depended_by(&dep, point);
            }
        }
        if let Some(up) = self.depended_by.remove(point) {
            for dep in up {
                self.remove_depend_on(&dep, point);
            }
        }
    }

    /// Replace a point with another
    pub fn replace_point<Q, R>(&mut self, from: &Q, to: &R) -> DependencyResult<()>
    where
        Q: Eq + Hash + PartialEq<R> + Clone,
        R: Eq + Hash + Clone,
        T: Borrow<Q> + Borrow<R> + From<Q> + From<R>,
    {
        if from == to {
            return Ok(());
        }
        if self.exists(to) {
            return Err(DependencyError::ReplaceToExistingPoint);
        }

        if let Some(down) = self.depends_on.remove(from) {
            for dep in down.iter() {
                let deps_of_dep = self.depended_by.get_mut(dep).unwrap();
                deps_of_dep.remove(from);
                deps_of_dep.insert(to.clone().into());
            }
            self.depends_on.insert(to.clone().into(), down);
        }
        if let Some(up) = self.depended_by.remove(from) {
            for dep in up.iter() {
                let deps_of_dep = self.depends_on.get_mut(dep).unwrap();
                deps_of_dep.remove(from);
                deps_of_dep.insert(to.clone().into());
            }
            self.depended_by.insert(to.clone().into(), up);
        }
        Ok(())
    }

    /// Check if `downstream` depends on `upstream` directly or indirectly
    pub fn is_dependent_recursively<Q, R>(&self, downstream: &Q, upstream: &R) -> bool
    where
        Q: Eq + Hash + PartialEq<R>,
        R: Eq + Hash + PartialEq<Q> + Clone,
        T: Borrow<Q> + Borrow<R> + Into<R>,
    {
        if downstream == upstream || self.is_leaf(upstream) || self.is_root(downstream) {
            return false;
        }

        let mut visited = HashSet::new();
        let mut stack = vec![upstream.clone()];

        while let Some(current) = stack.pop() {
            if &current == downstream {
                return true;
            }

            if visited.insert(current.clone())
                && let Some(dependencies) = self.depends_on(&current)
            {
                for dep in dependencies {
                    stack.push(dep.clone().into());
                }
            }
        }

        false
    }

    /// Get all points that depend on the given point directly or indirectly
    pub fn depends_on_recursively<Q>(&self, point: &Q) -> HashSet<T>
    where
        T: Borrow<Q> + Into<Q>,
        Q: Eq + Hash + Clone,
    {
        let mut result = HashSet::new();
        let mut stack = vec![point.clone()];

        while let Some(current) = stack.pop() {
            if let Some(deps) = self.depends_on.get(&current) {
                for dep in deps {
                    if result.insert(dep.clone()) {
                        stack.push(dep.clone().into());
                    }
                }
            }
        }
        result
    }

    /// Get all points that the given point depends on directly or indirectly
    pub fn depended_by_recursively<Q>(&self, point: &Q) -> HashSet<T>
    where
        T: Borrow<Q> + Into<Q>,
        Q: Eq + Hash + Clone,
    {
        let mut result = HashSet::new();
        let mut stack = vec![point.clone()];

        while let Some(current) = stack.pop() {
            if let Some(deps) = self.depended_by.get(&current) {
                for dep in deps {
                    if result.insert(dep.clone()) {
                        stack.push(dep.clone().into());
                    }
                }
            }
        }
        result
    }
}
