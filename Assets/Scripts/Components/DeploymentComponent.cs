using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Numerics;
using Assets.Scripts.Core;
using UnityEngine;

namespace Assets.Scripts.Components.Core
{
    [Serializable]
    public class DeploymentComponentCore
    {
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

            // If your job is deploy
            // TODO: worldObject.dispatchReceivers[0] sucks!!! do something else...
            if (
                worldObject.dispatchReceivers[0].receiverVerb
                == DispatchComponentCore.Verbs.Deploy.ToString()
            )
            {
                // If you have no more of the target item, switch to retrieve
                if (
                    worldObject.resources.resources[
                        worldObject.dispatchReceivers[0].receiverSubject
                    ] == 0
                )
                {
                    worldObject.dispatchReceivers[0].receiverVerb =
                        DispatchComponentCore.Verbs.Retrieve.ToString();
                }
                // If you don't have a target position, exit early
                if (worldObject.dispatchReceivers[0].targetPosition == null)
                {
                    return new()
                    {
                        new() { { gameController.backref.TickCount, "no deploy target position" } },
                    };
                }
                // If the target isn't adjacent, exit early
                float distance = System.Numerics.Vector2.Distance(
                    worldObject.dispatchReceivers[0].worldObject.gridPosition,
                    worldObject.dispatchReceivers[0].targetPosition.Value
                );
                if (distance > 1.5)
                {
                    return new();
                    // {
                    //     new()
                    //     {
                    //         { gameController.backref.TickCount, "not close enough to deploy" },
                    //     },
                    // };
                }
                // If you have the target item, remove it from your resources
                if (
                    worldObject.resources.resources.ContainsKey(
                        worldObject.dispatchReceivers[0].receiverSubject
                    )
                    && worldObject.resources.resources[
                        worldObject.dispatchReceivers[0].receiverSubject
                    ] > 0
                )
                {
                    worldObject.resources.ConsumeResources(
                        worldObject.dispatchReceivers[0].receiverSubject,
                        1
                    );

                    // Then spawn the target item at the target position
                    gameController.queuedForSpawn.Add(
                        new SpawnQueueItem(
                            worldObject.dispatchReceivers[0].receiverSubject,
                            (int)worldObject.dispatchReceivers[0].targetPosition.Value.X,
                            (int)worldObject.dispatchReceivers[0].targetPosition.Value.Y,
                            targetType: worldObject.dispatchReceivers[0].dispatcher.receiverObject
                        )
                    );
                }
            }

            return new();
        }
    }
}

namespace Assets.Scripts.Components.Tests
{
    using System.Collections.Generic;
    using Assets.Scripts.Components.Core;
    using Assets.Scripts.Core;
    using Xunit;
    using Xunit.Abstractions;

    internal class FactoryGameContent : GameContent
    {
        public override Dictionary<string, Item> Items { get; } =
            new()
            {
                { "wood", new Item("wood", stackSize: 100) },
                {
                    "sawmill",
                    new Item(
                        "sawmill",
                        stackSize: 1,
                        craftTime: 10,
                        ingredients: new Dictionary<string, uint> { { "sawmill", 5 } }
                    )
                },
            };
    }

    public class DeploymentComponentCoreTest
    {
        private ITestOutputHelper testOutput;

        public DeploymentComponentCoreTest(ITestOutputHelper testOutputHelper)
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
            WorldObjectCore worldObject = new(null);
            ResourcesComponentCore resources = new(
                new FactoryGameContent(),
                weightCapacity: 100,
                volumeCapacity: 100
            );
            worldObject.resources = resources;
            DispatchReceiverComponentCore dispatchReceiver = new(
                new WorldObjectCore(null),
                DispatchComponentCore.Verbs.Retrieve.ToString(),
                "sawmill"
            );
            worldObject.dispatchReceivers = new() { dispatchReceiver };
            DeploymentComponentCore deploy = new();
            deploy.Tick(gameController, worldObject);
            Assert.True(true);
            this.testOutput.WriteLine("tested true");
        }
    }
}
