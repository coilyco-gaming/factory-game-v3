// Inserter components query every nearby tile, once a tick,
// and insert matching items into the owner's resource inventory.


namespace Assets.Scripts.Components.Core
{
    using System.Collections.Generic;
    using System.Linq;
    using Assets.Scripts.Core;
    using Assets.Scripts.WorldObjects.Core;

    public class InserterComponentCore
    {
        private ResourcesComponentCore resources;
        private string resourceType = "";
        private uint insertionRate = 0;

        public InserterComponentCore(
            ResourcesComponentCore resources = null,
            string resourceType = "",
            uint insertionRate = 0
        )
        {
            this.resources = resources ?? new ResourcesComponentCore(new GameContent(), 0, 0);
            this.resourceType = resourceType;
            this.insertionRate = insertionRate;
        }

        public void Insert(WorldObjectCore worldObject, GameControllerCore gameController)
        {
            // TODO: inserters consume power
            List<WorldObjectCore> localWorldObjects = gameController.GetAdjacentWorldObjects(
                worldObject.GridPosition
            );
            List<ResourcesComponentCore> localResources = localWorldObjects
                .Select(localWorldObject => localWorldObject.Resources)
                .ToList();

            foreach (
                ResourcesComponentCore localResource in localResources
                    ?? new List<ResourcesComponentCore>()
            )
            {
                try
                {
                    // TODO: pass in a flag to supress alerts
                    this.resources?.TakeResouces(
                        localResource ?? new ResourcesComponentCore(new GameContent(), 0, 0),
                        this.resourceType,
                        this.insertionRate
                    );
                }
                catch (ResourcesComponentCore.ResourceException)
                {
                    continue;
                }
            }
        }
    }
}

namespace Assets.Scripts.Components.Tests
{
    using System.Collections.Generic;
    using Assets.Scripts.Components.Core;
    using Assets.Scripts.Core;
    using Assets.Scripts.WorldObjects.Core;
    using Xunit;

    public class InserterComponentTest
    {
        public WorldObjectCore WorldObject(
            GameControllerCore gameController,
            System.Numerics.Vector2 gridPosition,
            List<InserterComponentCore> inserters = null,
            ResourcesComponentCore resources = null
        )
        {
            WorldObjectCore worldObject = new(null)
            {
                Inserters = inserters,
                Resources = resources,
                GridPosition = gridPosition,
            };
            worldObject.Guid = worldObject.CreateGuid();
            gameController.worldObjects[worldObject.GridPosition] = new()
            {
                [worldObject.Guid] = worldObject,
            };
            return worldObject;
        }

        [Fact]
        public void TestNullsAndZero()
        {
            GameControllerCore gameController = new() { worldObjects = new() };

            // object 0
            WorldObjectCore worldObject0 = this.WorldObject(
                gameController,
                new System.Numerics.Vector2(0, 0),
                new List<InserterComponentCore>() { new() }
            );

            // object 1
            WorldObjectCore worldObject1 = this.WorldObject(
                gameController,
                new System.Numerics.Vector2(1, 0),
                new List<InserterComponentCore>() { new() },
                null
            );

            // object 2
            WorldObjectCore worldObject2 = this.WorldObject(
                gameController,
                new System.Numerics.Vector2(0, 1),
                new List<InserterComponentCore>() { new() },
                null
            );

            // logic under test
            worldObject0.Inserters[0].Insert(worldObject0, gameController); // TODO: assign a "parent" on every component
            worldObject1.Inserters[0].Insert(worldObject1, gameController); // TODO: the parent is the world object
            worldObject2.Inserters[0].Insert(worldObject2, gameController);
        }

        [Fact]
        public void TestInsertCapacityOverflow()
        {
            GameControllerCore gameController = new();
            gameController.worldObjects ??=
                new Dictionary<System.Numerics.Vector2, Dictionary<string, WorldObjectCore>>();

            // object 0
            ResourcesComponentCore resources0 = new(new TestResourcesGameContent(), 1, 1);
            resources0.CreateResources("wood", 1);
            WorldObjectCore worldObject0 = new(null)
            {
                Inserters = new() { new(resourceType: "wood", insertionRate: 1) },
                Resources = resources0,
                GridPosition = new System.Numerics.Vector2(0, 0),
            };
            gameController.worldObjects[worldObject0.GridPosition][worldObject0.Guid] =
                worldObject0;

            // object 1
            ResourcesComponentCore resources1 = new(new TestResourcesGameContent(), 1, 1);
            resources1.CreateResources("wood", 1);
            WorldObjectCore worldObject1 = new(null)
            {
                Inserters = new() { new(resourceType: "wood", insertionRate: 1) },
                Resources = resources1,
                GridPosition = new System.Numerics.Vector2(1, 0),
            };
            gameController.worldObjects[worldObject1.GridPosition][worldObject1.Guid] =
                worldObject1;

            // logic under test
            worldObject0.Inserters[0].Insert(worldObject0, gameController);
            worldObject1.Inserters[0].Insert(worldObject1, gameController);

            // assertions
            Assert.Equal(1u, worldObject0.Resources.Resources["wood"]);
            Assert.Equal(1u, worldObject1.Resources.Resources["wood"]);
        }

        [Fact]
        public void TestInsert()
        {
            GameControllerCore gameController = new();
            gameController.worldObjects ??=
                new Dictionary<System.Numerics.Vector2, Dictionary<string, WorldObjectCore>>();

            // object 0
            ResourcesComponentCore resources0 = new(new TestResourcesGameContent(), 2, 2);
            resources0.CreateResources("wood", 1);
            WorldObjectCore worldObject0 = new(null)
            {
                Inserters = new() { new(resourceType: "wood", insertionRate: 1) },
                Resources = resources0,
                GridPosition = new System.Numerics.Vector2(0, 0),
            };
            gameController.worldObjects[worldObject0.GridPosition][worldObject0.Guid] =
                worldObject0;

            // object 1
            ResourcesComponentCore resources1 = new(new TestResourcesGameContent(), 2, 2);
            resources1.CreateResources("wood", 1);
            WorldObjectCore worldObject1 = new(null)
            {
                Inserters = new() { new(resourceType: "wood", insertionRate: 1) },
                Resources = resources1,
                GridPosition = new System.Numerics.Vector2(1, 0),
            };
            gameController.worldObjects[worldObject1.GridPosition][worldObject1.Guid] =
                worldObject1;

            // logic under test
            worldObject0.Inserters[0].Insert(worldObject0, gameController);

            // assertions
            Assert.Equal(2u, worldObject0.Resources.Resources["wood"]);
            Assert.Equal(0u, worldObject1.Resources.Resources["wood"]);
        }

        [Fact]
        public void TestInsertMultiple()
        {
            GameControllerCore gameController = new();
            gameController.worldObjects ??=
                new Dictionary<System.Numerics.Vector2, Dictionary<string, WorldObjectCore>>();

            // object 0
            ResourcesComponentCore resources0 = new(new TestResourcesGameContent(), 3, 3);
            resources0.CreateResources("wood", 1);
            WorldObjectCore worldObject0 = new(null)
            {
                Inserters = new() { new(resourceType: "wood", insertionRate: 1) },
                Resources = resources0,
                GridPosition = new System.Numerics.Vector2(0, 0),
            };
            gameController.worldObjects[worldObject0.GridPosition][worldObject0.Guid] =
                worldObject0;

            // object 1
            ResourcesComponentCore resources1 = new(new TestResourcesGameContent(), 2, 2);
            resources1.CreateResources("wood", 1);
            WorldObjectCore worldObject1 = new(null)
            {
                Inserters = new() { new(resourceType: "wood", insertionRate: 1) },
                Resources = resources1,
                GridPosition = new System.Numerics.Vector2(1, 0),
            };
            gameController.worldObjects[worldObject1.GridPosition][worldObject1.Guid] =
                worldObject1;

            // object 2
            ResourcesComponentCore resources2 = new(new TestResourcesGameContent(), 2, 2);
            resources1.CreateResources("wood", 1);
            WorldObjectCore worldObject2 = new(null)
            {
                Inserters = new() { new(resourceType: "wood", insertionRate: 1) },
                Resources = resources2,
                GridPosition = new System.Numerics.Vector2(0, 1),
            };
            gameController.worldObjects[worldObject2.GridPosition][worldObject2.Guid] =
                worldObject2;

            // logic under test
            worldObject0.Inserters[0].Insert(worldObject0, gameController);

            // assertions
            Assert.Equal(3u, worldObject0.Resources.Resources["wood"]);
            Assert.Equal(0u, worldObject1.Resources.Resources["wood"]);
            Assert.Equal(0u, worldObject1.Resources.Resources["wood"]);
        }
    }
}
