use std::{
    any::{Any, TypeId},
    collections::HashMap,
    marker::PhantomData,
};

fn main() {
    let mut scheduler = Scheduler::new();

    scheduler.add_system(sample_system);
    scheduler.add_resource(512i32);

    scheduler.run();
}

fn sample_system(arg1: Res<i32>, arg2: Res<i64>) {
    println!("passed to sample system: {} and {}", arg1.value, arg2.value)
}

struct Scheduler {
    systems: Vec<StoredSystem>,
    resources: HashMap<TypeId, Box<dyn Any>>,
}

impl Scheduler {
    fn new() -> Self {
        Self {
            systems: vec![],
            resources: HashMap::default(),
        }
    }

    fn run(&mut self) {
        for system in self.systems.iter_mut() {
            system.run(&mut self.resources);
        }
    }

    fn add_system<I, S: System + 'static>(&mut self, system: impl IntoSystem<I, System = S>) {
        self.systems.push(Box::new(system.into_system()));
    }

    fn add_resource<R: 'static>(&mut self, resource: R) {
        self.resources.insert(TypeId::of::<R>(), Box::new(resource));
    }
}

type StoredSystem = Box<dyn System>;

trait System {
    fn run(&mut self, resources: &mut HashMap<TypeId, Box<dyn Any>>);
}

struct FunctionSystem<Input, F> {
    f: F,
    marker: PhantomData<fn() -> Input>,
}

trait IntoSystem<Input> {
    type System: System;

    fn into_system(self) -> Self::System;
}

trait SystemParam {
    type Item<'new>;

    fn retrieve<'r>(resources: &'r HashMap<TypeId, Box<dyn Any>>) -> Self::Item<'r>;
}

struct Res<'a, T: 'static> {
    value: &'a T,
}

impl<'res, T: 'static> SystemParam for Res<'res, T> {
    type Item<'new> = Res<'new, T>;

    fn retrieve<'r>(resources: &'r HashMap<TypeId, Box<dyn Any>>) -> Self::Item<'r> {
        Res {
            value: resources
                .get(&TypeId::of::<T>())
                .unwrap()
                .downcast_ref()
                .unwrap(),
        }
    }
}

impl<F, T1: SystemParam, T2: SystemParam> System for FunctionSystem<(T1, T2), F>
where
    for<'a, 'b> &'a mut F:
        FnMut(T1, T2) + FnMut(<T1 as SystemParam>::Item<'b>, <T2 as SystemParam>::Item<'b>),
{
    fn run(&mut self, resources: &mut HashMap<TypeId, Box<dyn Any>>) {
        fn call_inner<T1, T2>(mut f: impl FnMut(T1, T2), _0: T1, _1: T2) {
            f(_0, _1)
        }

        let _0 = T1::retrieve(resources);
        let _1 = T2::retrieve(resources);

        call_inner(&mut self.f, _0, _1);
    }
}

impl<F: FnMut(T1, T2), T1: SystemParam, T2: SystemParam> IntoSystem<(T1, T2)> for F
where
    for<'a, 'b> &'a mut F:
        FnMut(T1, T2) + FnMut(<T1 as SystemParam>::Item<'b>, <T2 as SystemParam>::Item<'b>),
{
    type System = FunctionSystem<(T1, T2), Self>;

    fn into_system(self) -> Self::System {
        FunctionSystem {
            f: self,
            marker: Default::default(),
        }
    }
}
