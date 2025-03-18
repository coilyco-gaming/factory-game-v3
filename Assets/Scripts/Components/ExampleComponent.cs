using System;
using System.Collections.Generic;
using System.Diagnostics;
using Assets.Scripts.Core;
using UnityEngine;

namespace Assets.Scripts.Components.Core
{
    [Serializable]
    public class ExampleComponentCore
    {
        private WorldObjectCore worldObject;

        public ExampleComponentCore(WorldObjectCore worldObject)
        {
            this.worldObject = worldObject;
        }

        public List<Dictionary<uint, string>> Tick(GameControllerCore gameController)
        {
            using Activity activity = gameController.backref.ActivitySource.StartActivity(
                this.GetType().Name
            );
            activity.SetTag("WorldObjectType", this.worldObject.worldObjectType);
            activity.SetTag("tick", gameController.backref.TickCount);
            return new();
        }
    }
}

namespace Assets.Scripts.Components.Tests
{
    using System.Diagnostics;
    using Assets.Scripts.Components.Core;
    using Assets.Scripts.Components.Unity;
    using OpenTelemetry;
    using OpenTelemetry.Resources;
    using OpenTelemetry.Trace;
    using Xunit;
    using Xunit.Abstractions;

    internal class ExampleGameController : IGameController
    {
        public uint TickCount { get; set; } = 0;
        public ActivitySource ActivitySource
        {
            get
            {
                ResourceBuilder resourceBuilder = ResourceBuilder
                    .CreateDefault()
                    .AddService("ExampleGameController");

                Sdk.CreateTracerProviderBuilder()
                    .SetResourceBuilder(resourceBuilder)
                    .AddSource("ExampleGameController")
                    .Build();

                return new("ExampleGameController");
            }
            set { }
        }
        public SpriteMapComponent Map { get; set; }

        public void QueueForMovement(MovementQueueItem movementQueueItem) { }

        public void QueueForDeletion(DeletionQueueItem deletionQueueItem) { }

        public void QueueForSpawn(SpawnQueueItem spawnQueueItem) { }
    }

    public class ExampleComponentCoreTest
    {
        private ITestOutputHelper testOutput;

        public ExampleComponentCoreTest(ITestOutputHelper testOutputHelper)
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
            ExampleComponentCore example = new(new WorldObjectCore(null));
            example.Tick(gameController);
            Assert.True(true);
            this.testOutput.WriteLine("tested true");
        }
    }
}
