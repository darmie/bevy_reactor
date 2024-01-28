# Things we need to demonstrate:

* Callbacks
* Verify Razing / Despawning doesn't leak
* Reacting to signals
* Setting signals in event handlers
* Reactive hover hook
* Reactive focus hook

# Other stuff

* No-arg .bind().
* Derived Signals
* Memo
* Effect
* Cleanup Handlers
* Scoped values
* Access to ECS components
* Access to owner entity
* use_element_rect hook
* transition hooks

# Obsidian

* Button
* Slider
* Flex
* etc...

## Notes on fine-grained

Tracking contexts. Start with create_memo:

    -- creates a tracking scope that is owned by the current context, like Solid
    -- it's a map of entity, component like before. Or resource.
    -- is it an entity?

DOM construction:

    let visible = create_memo(|| {

    });

#[derive(Component)]
struct Memo {
    value: Box<dyn Any>,
    deps_changed: bool,
}

#[derive(Component)]
struct Effect {
    action: fn()
}

create_memo:
    creates a tracking entity
    sets the current context to that entity
    attaches a "mark changed" action as the closure's action
    attaches the closure as the "recompute"
    runs the code
    restores the context

create_effect:
    creates a tracking entity
    sets the current context to that entity
    attaches the closure as that entity's action
    runs the code
    restores the context

element:
    children:
        each child is an "effect"
            effect has:
                a tracking scope
        child regens its nodes by calling build
        marks parent as needing re-flattening
    components:
        each component is an effect

view:
    view.build:
        optionally create tracking scope and effect function
        run effect, modify nodes

children are `Into<View>`.
components are `Into<ComponentBuilder>`

Constraint: tracking contexts need to be components, because that's the only way we
can iterate them. That means any reactive scope has to be an entity.

If views are entities:

* element needs a list of entities as children
* lifecycle:
    * init: creates dom and sets up effects
    * react: run effects and update dom
        * Need to notify owner
    * views only: assemble
    * cleanup: removes dom and cancels effects
    * despawn: deletes child views

If bundle providers are entities:
    element needs a list of entities as bundle providers
