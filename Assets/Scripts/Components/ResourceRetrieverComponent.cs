// Override resource reservations to pull a product out of
// its dispatchers resource inventory when it has become adjacent.

using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Linq;
using Assets.Scripts.Core;
using UnityEngine;

namespace Assets.Scripts.Components.Core
{
    [Serializable]
    public class ResourceRetrieverCore
    {
        private string targetResource;
        private uint quantity;

        public ResourceRetrieverCore(
            GameContent gameContent,
            string targetResource,
            uint quantity = 1
        )
        {
            this.targetResource = targetResource;
            this.quantity = quantity; // TODO: make this stack size
        }

        public List<Dictionary<uint, string>> Tick(
            GameControllerCore gameController,
            WorldObjectCore worldObject
        )
        {
            using Activity activity = gameController.backref.ActivitySource.StartActivity(
                this.GetType().Name
            );
            activity.SetTag("WorldObjectType", worldObject.worldObjectType);
            activity.SetTag("tick", gameController.backref.TickCount);
            activity.SetParentId(gameController.backref.WorldObjectTickActivity.Id);

            // TODO: worldObject.dispatchReceivers[0] is bad
            // Only get resources if we are in retrieve or collect mode
            bool weAreInRetrieveMode =
                worldObject.dispatchReceivers[0].receiverVerb
                == DispatchComponentCore.Verbs.Retrieve.ToString();
            bool weAreInCollectMode =
                worldObject.dispatchReceivers[0].receiverVerb
                == DispatchComponentCore.Verbs.Collect.ToString();
            if (!weAreInRetrieveMode && !weAreInCollectMode)
            {
                return new();
                // {
                //     new()
                //     {
                //         { gameController.backref.TickCount, "not in retrieve or collect mode" },
                //     },
                // };
            }

            // Only retrieve resources if we are adjacent to the dispatcher
            bool dispatcherIsAdjacent =
                worldObject.dispatchReceivers[0].targetPosition != null
                && System.Numerics.Vector2.Distance(
                    worldObject.gridPosition,
                    worldObject.dispatchReceivers[0].targetPosition.Value
                ) < 1.5;
            if (!dispatcherIsAdjacent)
            {
                return new();
                // {
                //     new() { { gameController.backref.TickCount, "dispatcher is not adjacent" } },
                // };
            }

            // Check if we already have some of the target resource
            bool weHaveTargetResource =
                worldObject.resources.resources.GetValueOrDefault(this.targetResource, 0u)
                > this.quantity;
            if (weHaveTargetResource)
            {
                return new();
                // {
                //     new() { { gameController.backref.TickCount, "we have resource to retrieve" } },
                // };
            }

            // Get all of the objects are the target position that might have the resources we want
            List<WorldObjectCore> targetWorldObjects = gameController
                .worldObjects.GetValueOrDefault(
                    worldObject.dispatchReceivers[0].targetPosition.Value
                )
                .Where(thisWorldObject => thisWorldObject.Value.resources != null)
                .Select(thisWorldObject => thisWorldObject.Value)
                .ToList();

            try
            {
                // Ask every object at the target position to give us the resources we want
                // This is a shakedown! Everyone give up your resources!
                foreach (WorldObjectCore targetWorldObject in targetWorldObjects)
                {
                    worldObject.resources.RetrieveResources(
                        targetWorldObject.resources,
                        this.targetResource,
                        this.quantity
                    );
                }
            }
            catch (ResourcesComponentCore.ResourceException) { }

            return new();
        }
    }
}

namespace Assets.Scripts.Components.Tests
{
    using System.Collections.Generic;
    using Assets.Scripts.Components.Core;
    using Xunit;
    using Xunit.Abstractions;

    internal class TestResourceReceiverGameContent : GameContent
    {
        public override Dictionary<string, Item> Items { get; } =
            new()
            {
                { "wood", new Item("wood", stackSize: 100) },
                {
                    "planks",
                    new Item(
                        "planks",
                        stackSize: 10,
                        craftTime: 3,
                        ingredients: new Dictionary<string, uint> { { "wood", 5 } }
                    )
                },
            };
    }

    public class ResourceRetrieverCoreTest
    {
        private ITestOutputHelper testOutput;

        public ResourceRetrieverCoreTest(ITestOutputHelper testOutputHelper)
        {
            this.testOutput = testOutputHelper;
        }

        [Fact]
        public void TestTrue()
        {
            GameControllerCore gameController = new()
            {
                backref = new ExampleGameController(),
                worldObjects = new(),
            };

            // Receiver
            ResourcesComponentCore receiverResources = new(
                new TestResourceReceiverGameContent(),
                100,
                100
            );
            WorldObjectCore receiverWorldObject = new(null) { resources = receiverResources };
            ResourceRetrieverCore resourceReceiver = new(
                new TestResourceReceiverGameContent(),
                "planks",
                1
            );
            receiverWorldObject.dispatchReceivers = new() { new DispatchReceiverComponentCore() };

            // Logic under test
            resourceReceiver.Tick(gameController, receiverWorldObject);
            Assert.True(true);
            this.testOutput.WriteLine("tested true");
        }

        [Fact]
        public void TestRetrieve()
        {
            GameControllerCore gameController = new()
            {
                backref = new ExampleGameController(),
                worldObjects = new(),
            };

            // Dispatcher
            WorldObjectCore dispatcherWorldObject = new(null) { gridPosition = new(0, 0) };
            ResourcesComponentCore dispatcherResources = new(
                new TestResourceReceiverGameContent(),
                100,
                100
            );
            dispatcherWorldObject.resources = dispatcherResources;
            dispatcherResources.CreateResources("planks", 1);
            gameController.worldObjects[dispatcherWorldObject.gridPosition] = new()
            {
                { "uuid-0", dispatcherWorldObject },
            };
            dispatcherWorldObject.dispatchers = new()
            {
                new DispatchComponentCore(
                    new TestResourceReceiverGameContent(),
                    DispatchComponentCore.Verbs.Retrieve.ToString(),
                    "planks",
                    DispatchComponentCore.Keywords.Me.ToString()
                ),
            };

            // Receiver
            ResourcesComponentCore receiverResources = new(
                new TestResourceReceiverGameContent(),
                100,
                100
            );
            WorldObjectCore receiverWorldObject = new(null)
            {
                resources = receiverResources,
                gridPosition = new(0, 1),
            };
            ResourceRetrieverCore resourceReceiver = new(
                new TestResourceReceiverGameContent(),
                "planks",
                1
            );
            receiverWorldObject.dispatchReceivers = new()
            {
                new DispatchReceiverComponentCore(
                    DispatchComponentCore.Verbs.Retrieve.ToString(),
                    "planks"
                ),
            };
            receiverWorldObject.dispatchReceivers[0].targetPosition = new(0, 0);
            receiverWorldObject.dispatchReceivers[0].dispatcher = dispatcherWorldObject.dispatchers[
                0
            ];
            gameController.worldObjects[receiverWorldObject.gridPosition] = new()
            {
                { "uuid-1", receiverWorldObject },
            };

            // Logic under test
            resourceReceiver.Tick(gameController, receiverWorldObject);

            // Assertions
            Assert.Equal(receiverResources.resources["planks"], 1u);
        }

        // [Fact]
        // public void TestDoesNotDuplicate()
        // {
        //     GameControllerCore gameController = new()
        //     {
        //         backref = new ExampleGameController(),
        //         worldObjects = new(),
        //     };

        //     // Dispatcher
        //     WorldObjectCore dispatcherWorldObject = new(null);
        //     ResourcesComponentCore dispatcherResources = new(
        //         new TestResourceReceiverGameContent(),
        //         100,
        //         100
        //     );
        //     dispatcherWorldObject.resources = dispatcherResources;
        //     dispatcherResources.CreateResources("planks", 1);
        //     DispatchComponentCore dispatcher = new(
        //         new TestResourceReceiverGameContent(),
        //         DispatchComponentCore.Verbs.Retrieve.ToString(),
        //         "planks",
        //         DispatchComponentCore.Keywords.Me.ToString()
        //     );

        //     // Receiver
        //     ResourcesComponentCore receiverResources = new(
        //         new TestResourceReceiverGameContent(),
        //         100,
        //         100
        //     );
        //     BatteryComponentCore receiverBattery = new(100, 100);
        //     WorldObjectCore receiverWorldObject = new(null)
        //     {
        //         resources = receiverResources,
        //         battery = receiverBattery,
        //         gridPosition = new(0, 1),
        //     };
        //     DispatchReceiverComponentCore receiverDispatcher = new(
        //         DispatchComponentCore.Verbs.Retrieve.ToString(),
        //         "planks"
        //     )
        //     {
        //         dispatcher = dispatcher,
        //         targetPosition = new(0, 1),
        //     };
        //     receiverWorldObject.dispatchReceivers = new() { receiverDispatcher };
        //     receiverWorldObject.resources = receiverResources;

        //     ResourceRetrieverCore resourceReceiver = new(
        //         new TestResourceReceiverGameContent(),
        //         "planks"
        //     );

        //     // Logic under test
        //     resourceReceiver.Tick(gameController, receiverWorldObject);
        //     resourceReceiver.Tick(gameController, receiverWorldObject);
        //     resourceReceiver.Tick(gameController, receiverWorldObject);

        //     // Assertions
        //     Assert.Equal(receiverResources.resources["planks"], 1u);
        // }
    }
}
