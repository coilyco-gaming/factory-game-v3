using System.Numerics;
using Assets.Scripts.Core;

namespace Assets.Scripts.Components.Core
{
    public class DeploymentComponentCore
    {
        private ResourcesComponentCore resources;
        private DispatchReceiverComponentCore dispatchReceiver;

        public DeploymentComponentCore(
            ResourcesComponentCore resources,
            DispatchReceiverComponentCore dispatchReceiver
        )
        {
            this.resources = resources;
            this.dispatchReceiver = dispatchReceiver;
        }

        public void Tick(GameControllerCore gameController)
        {
            // If your job is deploy
            if (this.dispatchReceiver.receiverVerb == DispatchComponentCore.Verbs.Deploy.ToString())
            {
                // If you have no more of the target item, switch to retrieve
                if (this.resources.resources[this.dispatchReceiver.receiverSubject] == 0)
                {
                    this.dispatchReceiver.receiverVerb =
                        DispatchComponentCore.Verbs.Retrieve.ToString();
                }
                // If you don't have a target position, exit early
                if (this.dispatchReceiver.targetPosition == null)
                {
                    return;
                }
                // If the target isn't adjacent, exit early
                float distance = Vector2.Distance(
                    this.dispatchReceiver.worldObject.gridPosition,
                    this.dispatchReceiver.targetPosition.Value
                );
                if (distance > 1.5)
                {
                    return;
                }
                // If you have the target item, remove it from your resources
                if (
                    this.resources.resources.ContainsKey(this.dispatchReceiver.receiverSubject)
                    && this.resources.resources[this.dispatchReceiver.receiverSubject] > 0
                )
                {
                    this.resources.ConsumeResources(this.dispatchReceiver.receiverSubject, 1);

                    // Then spawn the target item at the target position
                    gameController.queuedForSpawn.Add(
                        new SpawnQueueItem(
                            this.dispatchReceiver.receiverSubject,
                            (int)this.dispatchReceiver.targetPosition.Value.X,
                            (int)this.dispatchReceiver.targetPosition.Value.Y,
                            targetType: this.dispatchReceiver.dispatcher.receiverObject
                        )
                    );
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
            GameControllerCore gameController = new();
            ResourcesComponentCore resources = new(
                new FactoryGameContent(),
                weightCapacity: 100,
                volumeCapacity: 100
            );
            DispatchReceiverComponentCore dispatchReceiver = new(
                new WorldObjectCore(null),
                resources,
                DispatchComponentCore.Verbs.Retrieve.ToString(),
                "sawmill"
            );
            DeploymentComponentCore deploy = new(resources, dispatchReceiver);
            deploy.Tick(gameController);
            Assert.True(true);
            this.testOutput.WriteLine("tested true");
        }
    }
}
