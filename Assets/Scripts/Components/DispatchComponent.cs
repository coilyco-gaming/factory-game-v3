namespace Assets.Scripts.Components.Core
{
    using System;
    using System.Collections.Generic;
    using System.Diagnostics;
    using System.Linq;
    using Assets.Scripts.Core;
    using UnityEngine;

    [Serializable]
    public class DispatchComponentCore
    {
        private uint deliveryResourceBufferMultiplier = 4;

        // Example descriptions:
        //  - Retrieve power lines from me
        //  - Deploy mining drill to aluminum ore
        //  - Collect coal to me
        private string DescriptionToOrFrom =>
            this.receiverVerb == DispatchComponentCore.Verbs.Retrieve.ToString()
            || this.receiverVerb == DispatchComponentCore.Verbs.Collect.ToString()
                ? "from"
                : "to";
        private string DescriptionSubject => Util.HumanizedString(this.receiverSubject);
        private string DescriptionObject => Util.HumanizedString(this.receiverObject);
        public string Description =>
            $"{this.receiverVerb} {this.DescriptionSubject} {this.DescriptionToOrFrom} {this.DescriptionObject}".ToLower();

        public WorldObjectCore worldObject;
        public DispatchReceiverComponentCore receiver;
        private BatteryComponentCore battery;
        private ResourcesComponentCore resources;
        private GameContent gameContent;
        public string receiverVerb = "VERB";
        public string receiverSubject = "SUBJECT";
        public string receiverObject = "OBJECT";

        public enum Verbs
        {
            // Collect <=> Deliver
            Collect,
            Deliver,

            // Retrieve <=> Deploy
            Retrieve,
            Deploy,
        }

        public enum Keywords
        {
            Me,
        }

        public DispatchComponentCore(
            GameContent gameContent,
            WorldObjectCore worldObject,
            string receiverVerb,
            string receiverSubject,
            string receiverObject
        )
        {
            this.gameContent = gameContent;
            this.worldObject = worldObject;
            this.receiverVerb = receiverVerb;
            this.receiverSubject = receiverSubject;
            this.receiverObject = receiverObject;
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

            // TODO: not this
            this.worldObject = worldObject;

            // If the dispatch has already been assigned, then skip
            if (this.receiver != null)
            {
                return new()
                {
                    new()
                    {
                        {
                            gameController.backref.TickCount,
                            $"{this.Description}, assigned to {this.receiver.worldObject.guid}"
                        },
                    },
                };
            }

            // If dispatch goal is deliver to me
            // then skip if I have more than a X stacks of the item
            if (
                this.receiverVerb == DispatchComponentCore.Verbs.Deliver.ToString()
                && this.receiverObject == DispatchComponentCore.Keywords.Me.ToString()
                && worldObject.resources.resources.GetValueOrDefault(this.receiverSubject)
                    > this.deliveryResourceBufferMultiplier
                        * this.gameContent.Items[this.receiverSubject].StackSize
            )
            {
                return new()
                {
                    new()
                    {
                        {
                            gameController.backref.TickCount,
                            $"{this.Description}, no more required"
                        },
                    },
                };
            }

            // If dispatch goal is retrieve or collect from me
            // then skip if I don't have any of the item
            if (
                (
                    this.receiverVerb == DispatchComponentCore.Verbs.Retrieve.ToString()
                    || this.receiverVerb == DispatchComponentCore.Verbs.Collect.ToString()
                )
                && this.receiverObject == DispatchComponentCore.Keywords.Me.ToString()
                && worldObject.resources.resources.GetValueOrDefault(this.receiverSubject) == 0
            )
            {
                return new()
                {
                    new()
                    {
                        {
                            gameController.backref.TickCount,
                            $"{this.Description}, not enough available"
                        },
                    },
                };
            }

            // Only dispatch if there's an empty adjacent tile
            bool hasEmptyAdjacent = false;
            foreach (
                System.Numerics.Vector2 adjacentTile in GameControllerCore.GetAdjacentPositions(
                    worldObject.gridPosition
                )
            )
            {
                if (!gameController.worldObjects.ContainsKey(adjacentTile))
                {
                    hasEmptyAdjacent = true;
                    break;
                }
                if (
                    gameController.worldObjects.ContainsKey(adjacentTile)
                    && gameController
                        .worldObjects[adjacentTile]
                        .Where(worldObject => !worldObject.Value.passThrough)
                        .Where(worldObject => !worldObject.Value.mobile)
                        .ToList()
                        .Count == 0
                )
                {
                    hasEmptyAdjacent = true;
                    break;
                }
            }
            if (!hasEmptyAdjacent)
            {
                return new()
                {
                    new()
                    {
                        {
                            gameController.backref.TickCount,
                            $"{this.Description}, no empty adjacent tile"
                        },
                    },
                };
            }

            // Abort early if the battery is empty
            try
            {
                worldObject.battery.Energy -= 1;
            }
            catch (BatteryComponentCore.BatteryCapacityException)
            {
                return new();
            }

            // Acqiure list of target locations
            //  - If the receiver is me, then return the current world object location
            //  - If the receiver is not me, then return the list of world objects
            //    whose world object type match the receiver object
            //
            // Examples:
            //  - Deploy mining drill to aluminum ore
            //    targetLocations = < aluminum ore grid positions >
            //  - Deliver aluminum ore to me, Collect aluminum ore from me
            //    targetLocations = < current world object grid position >
            List<System.Numerics.Vector2> targetLocations =
                this.receiverObject == DispatchComponentCore.Keywords.Me.ToString()
                    ? new List<System.Numerics.Vector2> { worldObject.gridPosition }
                    : gameController
                        .worldObjects
                        // For world object locations that do not already have a dispatch
                        .SelectMany(worldObjects => worldObjects.Value)
                        // For world objects that contain the target type
                        .Where(thisWorldObject =>
                            thisWorldObject.Value.worldObjectType == this.receiverObject
                        )
                        // Order by distance to the current world object
                        .OrderBy(thisWorldObject =>
                            System.Numerics.Vector2.Distance(
                                thisWorldObject.Value.GridPosition,
                                worldObject.GridPosition
                            )
                        )
                        // Select the grid position of the target world objects
                        .Select(worldObject => worldObject.Value.GridPosition)
                        .ToList();

            // If dispatch verb is deploy, filter out target locations
            // that are already occupied by the same dispatch subject

            if (this.receiverVerb == DispatchComponentCore.Verbs.Deploy.ToString())
            {
                targetLocations = targetLocations
                    .Where(targetLocation =>
                        !gameController
                            .worldObjects[targetLocation]
                            .Any(worldObject =>
                                worldObject.Value.worldObjectType == this.receiverSubject
                            )
                    )
                    .ToList();
            }

            // Don't dispatch to target if there's already another dispatch
            // of the same type assigned to the same location
            List<DispatchComponentCore> dispatchers = gameController
                .worldObjects
                // For all world objects
                .SelectMany(worldObjects => worldObjects.Value)
                .Where(worldObject =>
                    // Where the world object has dispatchers
                    worldObject.Value.dispatchers != null
                )
                // For all dispatchers
                .SelectMany(worldObject => worldObject.Value.dispatchers)
                .Where(dispatcher =>
                    // Where the dispatcher is not null and is awaiting a target
                    dispatcher != null
                    && dispatcher.receiver != null
                    // Where the dispatcher Subject and verb match the dispatch
                    && dispatcher.receiverSubject == this.receiverSubject
                    && this.receiverVerb == dispatcher.receiverVerb
                )
                .ToList();

            // Remove target locations that already have a dispatcher assigned
            // to the same type of dispatch
            targetLocations = targetLocations
                .Where(targetLocation =>
                    !dispatchers.Any(dispatcher =>
                        dispatcher.receiver != null
                        && dispatcher.receiver.targetPosition == targetLocation
                    )
                )
                .ToList();

            // TODO: don't assign is target is not adjacent or there is a path to the target

            if (targetLocations.Count == 0)
            {
                return new()
                {
                    new()
                    {
                        {
                            gameController.backref.TickCount,
                            $"{this.Description}, no target found"
                        },
                    },
                };
            }

            // TODO: immobible recievers only match when they are adjacent
            // Get the first receiver awaiting a target
            DispatchReceiverComponentCore receiver = gameController
                .worldObjects
                // For all world objects
                .SelectMany(worldObjects => worldObjects.Value)
                .Where(worldObject =>
                    // Where the world object has dispatch receivers
                    worldObject.Value.dispatchReceivers != null
                )
                // For all dispatch receivers
                .SelectMany(worldObject => worldObject.Value.dispatchReceivers)
                .Where(receiver =>
                    // Where the receiver is not null and is awaiting a target
                    receiver != null
                    && receiver.dispatcher == null
                    // Where the receiver Subject and verb match the dispatch
                    && receiver.receiverSubject == this.receiverSubject
                    && this.receiverVerb == receiver.receiverVerb
                )
                // Order by distance to the current world object
                .OrderBy(receiver =>
                    System.Numerics.Vector2.Distance(
                        receiver.worldObject.GridPosition,
                        worldObject.GridPosition
                    )
                )
                .FirstOrDefault();

            if (receiver == null)
            {
                return new()
                {
                    new()
                    {
                        {
                            gameController.backref.TickCount,
                            $"{this.Description}, no receiver found"
                        },
                    },
                };
            }

            // Assign the target to the receiver
            receiver.QueueDispatch(this, targetLocations[0], gameController);
            this.receiver = receiver;
            return new();
        }
    }
}

namespace Assets.Scripts.Components.Tests
{
    using System.Collections.Generic;
    using System.Linq;
    using Assets.Scripts.Components.Core;
    using Assets.Scripts.Core;
    using Xunit;
    using Xunit.Abstractions;

    internal class TestDispatchGameContent : GameContent
    {
        public override Dictionary<string, Item> Items { get; } =
            new()
            {
                {
                    "aluminumBars",
                    new Item(
                        "aluminumBars",
                        stackSize: 10,
                        ingredients: new Dictionary<string, uint> { { "fakeOre", 5 } }
                    )
                },
                {
                    "MiningDrill",
                    new Item(
                        "MiningDrill",
                        stackSize: 1,
                        craftTime: 3,
                        ingredients: new Dictionary<string, uint> { { "aluminumBars", 5 } }
                    )
                },
            };
    }

    public class DispatchComponentTest
    {
        private ITestOutputHelper testOutput;

        public DispatchComponentTest(ITestOutputHelper testOutputHelper)
        {
            this.testOutput = testOutputHelper;
        }

        [Fact]
        public void TestTrue()
        {
            GameControllerCore gameController = new() { backref = new ExampleGameController() };
            WorldObjectCore worldObject = new(null);
            BatteryComponentCore battery = new(100);
            worldObject.battery = battery;
            ResourcesComponentCore resources = new(new TestDispatchGameContent(), 100, 100);
            worldObject.resources = resources;
            DispatchComponentCore dispatch = new(
                new TestDispatchGameContent(),
                worldObject,
                "Deploy",
                "MiningDrill",
                "fakeOre"
            );
            dispatch.Tick(gameController, worldObject);
            Assert.True(true);
        }

        [Fact]
        public void TestAssignTarget()
        {
            GameControllerCore gameController = new() { backref = new ExampleGameController() };
            WorldObjectCore HQWorldObject = new(null)
            {
                GridPosition = new System.Numerics.Vector2(0, 0),
            };

            BatteryComponentCore battery = new(100);
            ResourcesComponentCore dispatcherResources = new(
                new TestDispatchGameContent(),
                100,
                100
            );
            HQWorldObject.battery = battery;
            HQWorldObject.resources = dispatcherResources;
            DispatchComponentCore dispatch = new(
                new TestDispatchGameContent(),
                HQWorldObject,
                "Deploy",
                "MiningDrill",
                "fakeOre"
            );
            HQWorldObject.dispatchers = new List<DispatchComponentCore> { dispatch };

            WorldObjectCore targetWorldObject = new(null)
            {
                GridPosition = new System.Numerics.Vector2(1, 1),
                worldObjectType = "fakeOre",
            };

            WorldObjectCore receiverWorldObject = new(null)
            {
                GridPosition = new System.Numerics.Vector2(2, 2),
            };

            ResourcesComponentCore receiverResources = new(new TestDispatchGameContent(), 100, 100);
            receiverResources.CreateResources("MiningDrill", 1);
            receiverWorldObject.resources = receiverResources;

            DispatchReceiverComponentCore receiver = new(
                receiverWorldObject,
                "Deploy",
                "MiningDrill"
            );
            receiverWorldObject.dispatchReceivers = new() { receiver };
            Assert.Null(receiver.dispatcher);

            gameController.worldObjects[new System.Numerics.Vector2(0, 0)] = new()
            {
                { "uuid-1", HQWorldObject },
            };
            gameController.worldObjects[new System.Numerics.Vector2(1, 1)] = new()
            {
                { "uuid-2", targetWorldObject },
            };
            gameController.worldObjects[new System.Numerics.Vector2(2, 2)] = new()
            {
                { "uuid-3", receiverWorldObject },
            };

            dispatch.Tick(gameController, HQWorldObject);
            receiver.Tick(gameController, receiverWorldObject);
            Assert.NotNull(receiver.dispatcher);
        }
    }
}
