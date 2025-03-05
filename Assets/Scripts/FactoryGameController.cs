using System;
using System.Collections.Generic;
using System.Linq;
using Assets.Scripts.Components.Unity;
using Assets.Scripts.ScriptableObject;
using Assets.Scripts.WorldObjects.Core;
using TMPro;
using UnityEngine;
using UnityEngine.Rendering;
using UnityEngine.UI;

namespace Assets.Scripts.Unity
{
    public class FactoryGameController : GameController
    {
        public GameObject resetButton;
        public GameObject pauseButton;
        public GameObject StatusUILeft;
        public GameObject StatusUIRight;
        public uint HQOreBuffer = 5;
        public uint spawnAttempts = 5;
        public float oreSpawnFactor = 0.5f;
        public uint OreQuantityBase = 2000;
        public uint OreQuantityRange = 1000;
        public List<SpawnQueueItem> spawnQueueItems;
        private TextMeshProUGUI pauseTextComponent;
        private StatusUILeftComponent StatusUILeftComponent;
        private StatusUIRightComponent StatusUIRightComponent;

        public override void Start()
        {
            base.Start();

            this.Map = this.GetComponent<SpriteMapComponent>();
            this.Map.Instantiate(this.GetComponent<Canvas>());

            this.PlayerComponent = this.GetComponent<PlayerComponent>();
            this.PlayerComponent.Instantiate(this.Map.mapSize.x, this.Map.mapSize.y);

            this.StatusUILeftComponent = this.StatusUILeft.GetComponent<StatusUILeftComponent>();
            this.StatusUILeftComponent.Instantiate();

            this.StatusUIRightComponent = this.StatusUIRight.GetComponent<StatusUIRightComponent>();
            this.StatusUIRightComponent.Instantiate();

            Button resetComponent = this.resetButton.GetComponent<Button>();
            resetComponent.onClick.AddListener(this.Reset);

            Button pauseComponent = this.pauseButton.GetComponent<Button>();
            pauseComponent.onClick.AddListener(this.TogglePausePlay);

            this.pauseTextComponent = this.pauseButton.GetComponentInChildren<TextMeshProUGUI>();
            this.RenderPausePlay(true); // start paused

            this.Reset();
        }

        public override void Update()
        {
            base.Update();
            this.WriteStatusUILeft();
            this.WriteStatusUIRight();
        }

        protected override void Reset()
        {
            base.Reset();
            this.PlayerComponent.Reset();
            this.SpawnOres(FactoryGameContent.Resources.Iron.ToString());
            this.SpawnOres(FactoryGameContent.Resources.Copper.ToString());
            this.SpawnOres(FactoryGameContent.Resources.Coal.ToString());

            // Spawn some initial buildings
            //
            // C = Coal Plant
            // F = Factory
            // W = Storage Warehouse  <== simply stores things
            // X = Transfer Warehouse <== acts as a buffer between buildings
            // S = Radar
            // T = Truck
            //
            //   T T T T
            // R W F F F
            // R C   X X
            // R C F F F F <== the radar on the far left is our 0x 0y
            //   W W W W W
            //
            // Both coal plants have a warehouse beside them with a stockpile of coal.
            // There's 1 scanner for each resource: iron, copper, coal.
            //
            // The factories are chained like so:
            //      W W W W
            //      F F F F
            //      ^ ^ ^ ^
            //      | | | BuildingMaterials
            //      | | Motors
            //      | Circuits
            //      Frames
            //
            // The factories adjacent to the transfer array are in this order:
            //  - Factory
            //  - Warehouse
            //  - Coal Plant
            //
            // TODO: a proper HQ building that has enhanced capabilities


            this.Spawn(
                new SpawnQueueItem(
                    "CoalPlant",
                    x: 1,
                    y: 0,
                    xyCentered: true,
                    resources: new Dictionary<string, uint> { { "Coal", 100 } }
                )
            );

            this.Spawn(
                new SpawnQueueItem(
                    "CoalPlant",
                    x: 1,
                    y: 1,
                    xyCentered: true,
                    resources: new Dictionary<string, uint> { { "Coal", 100 } }
                )
            );

            this.Spawn(
                new SpawnQueueItem("Radar", x: 0, y: 0, xyCentered: true, targetType: "Iron")
            );

            this.Spawn(
                new SpawnQueueItem("Radar", x: 0, y: 1, xyCentered: true, targetType: "Copper")
            );

            this.Spawn(
                new SpawnQueueItem("Radar", x: 0, y: 2, xyCentered: true, targetType: "Coal")
            );

            this.Spawn(
                new SpawnQueueItem("Factory", x: 2, y: 0, xyCentered: true, targetType: "Frames")
            );

            this.Spawn(
                new SpawnQueueItem("Factory", x: 3, y: 0, xyCentered: true, targetType: "Circuits")
            );

            this.Spawn(
                new SpawnQueueItem("Factory", x: 4, y: 0, xyCentered: true, targetType: "Motors")
            );

            this.Spawn(
                new SpawnQueueItem(
                    "Factory",
                    x: 5,
                    y: 0,
                    xyCentered: true,
                    targetType: "BuildingMaterials"
                )
            );

            this.Spawn(
                new SpawnQueueItem("Factory", x: 2, y: 2, xyCentered: true, targetType: "Factory")
            );

            this.Spawn(
                new SpawnQueueItem(
                    "Factory",
                    x: 3,
                    y: 2,
                    xyCentered: true,
                    targetType: "StorageWarehouse"
                )
            );

            this.Spawn(
                new SpawnQueueItem("Factory", x: 4, y: 2, xyCentered: true, targetType: "CoalPlant")
            );

            this.Spawn(
                new SpawnQueueItem(
                    "StorageWarehouse",
                    x: 1,
                    y: -1,
                    xyCentered: true,
                    resources: new Dictionary<string, uint> { { "Coal", 5000 } }
                )
            );

            this.Spawn(
                new SpawnQueueItem(
                    "StorageWarehouse",
                    x: 1,
                    y: 2,
                    xyCentered: true,
                    resources: new Dictionary<string, uint> { { "Coal", 5000 } }
                )
            );

            this.Spawn(
                new SpawnQueueItem(
                    "StorageWarehouse",
                    x: 2,
                    y: -1,
                    xyCentered: true,
                    resources: new Dictionary<string, uint>
                    {
                        { "Stone", 4000 },
                        { "Iron", 2000 },
                        { "Copper", 1000 },
                    }
                )
            );

            this.Spawn(
                new SpawnQueueItem(
                    "StorageWarehouse",
                    x: 3,
                    y: -1,
                    xyCentered: true,
                    resources: new Dictionary<string, uint>
                    {
                        { "Stone", 4000 },
                        { "Iron", 2000 },
                        { "Copper", 1000 },
                    }
                )
            );

            this.Spawn(
                new SpawnQueueItem(
                    "StorageWarehouse",
                    x: 4,
                    y: -1,
                    xyCentered: true,
                    resources: new Dictionary<string, uint>
                    {
                        { "Stone", 4000 },
                        { "Iron", 2000 },
                        { "Copper", 1000 },
                    }
                )
            );

            this.Spawn(
                new SpawnQueueItem(
                    "StorageWarehouse",
                    x: 5,
                    y: -1,
                    xyCentered: true,
                    resources: new Dictionary<string, uint>
                    {
                        { "Stone", 4000 },
                        { "Iron", 2000 },
                        { "Copper", 1000 },
                    }
                )
            );

            this.Spawn(new SpawnQueueItem("TransferWarehouse", x: 3, y: 1, xyCentered: true));

            this.Spawn(new SpawnQueueItem("TransferWarehouse", x: 4, y: 1, xyCentered: true));

            this.Spawn(new SpawnQueueItem("Truck", x: 1, y: 3, xyCentered: true));
            this.Spawn(new SpawnQueueItem("Truck", x: 2, y: 3, xyCentered: true));
            this.Spawn(new SpawnQueueItem("Truck", x: 3, y: 3, xyCentered: true));
            this.Spawn(new SpawnQueueItem("Truck", x: 4, y: 3, xyCentered: true));

            this.readyForTicks = false;
        }

        private void SpawnOres(string objectName)
        {
            int oresToSpawn = (int)(
                Mathf.Sqrt(
                    (float)(Math.Pow(this.Map.mapSize.x, 2) + Math.Pow(this.Map.mapSize.y, 2))
                ) * this.oreSpawnFactor
            );
            for (int i = 0; i < oresToSpawn; i++)
            {
                for (int attempts = 0; attempts < this.spawnAttempts; attempts++)
                {
                    // Start by picking a random position
                    int x = this.random.Next(0, this.Map.mapSize.x);
                    int y = this.random.Next(0, this.Map.mapSize.y);

                    // Draw a triangle to ensure that our ore is at least 2 blocks away
                    uint xOffset = (uint)Math.Abs(x - (this.Map.mapSize.x / (float)2));
                    uint yOffset = (uint)Math.Abs(y - (this.Map.mapSize.y / (float)2));
                    double distance = Math.Sqrt(Math.Pow(xOffset, 2) + Math.Pow(yOffset, 2));
                    if (distance <= this.HQOreBuffer)
                    {
                        continue;
                    }
                    System.Numerics.Vector2 gridPosition = new(x, y);

                    // Only spawn if there isn't already in ore at this position
                    bool oreAtPosition =
                        this.GetWorldObjectsByPositionAndType(
                            gridPosition,
                            new List<string>
                            {
                                FactoryGameContent.Resources.Iron.ToString(),
                                FactoryGameContent.Resources.Copper.ToString(),
                                FactoryGameContent.Resources.Coal.ToString(),
                            }
                        ) != null;

                    // Get ore amount
                    float randomPercent = (float)this.random.Next(-100, 100) / 100;
                    int oreQuantityChange = (int)(randomPercent * this.OreQuantityRange);
                    uint oreQuantity = (uint)(this.OreQuantityBase + oreQuantityChange);

                    // Spawn ore
                    if (!oreAtPosition)
                    {
                        this.Spawn(
                            new SpawnQueueItem(
                                objectName,
                                x: x,
                                y: y,
                                resources: new Dictionary<string, uint>
                                {
                                    { objectName, oreQuantity },
                                }
                            )
                        );
                        break;
                    }
                }
            }
        }

        private void TogglePausePlay()
        {
            this.readyForTicks = !this.readyForTicks; // toggle
            this.RenderPausePlay(!this.readyForTicks); // if not ready, then render paused
        }

        private void RenderPausePlay(bool paused)
        {
            string playText = "Play";
            string pauseText = "Pause";
            this.pauseTextComponent.SetText(paused ? pauseText : playText);
            this.PlayerComponent.ToggleFogPosition(paused); // if paused, then move flow closer
        }

        private void WriteStatusUILeft()
        {
            // Update the UI state with whatever the player is looking at

            // Get the list of objects at the player's position
            System.Numerics.Vector2 position = this.PlayerComponent.GetGridPosition();

            // TODO: grab the nearby objects, not just the ones at the player's position
            List<WorldObjectCore> worldObjects =
                this?.core?.worldObjects?.GetValueOrDefault(position, null)?.Values.ToList()
                ?? new List<WorldObjectCore>();

            // Display the status data in the UI
            this.StatusUILeftComponent.Display(worldObjects);
        }

        private void WriteStatusUIRight()
        {
            // TODO: grab the nearby objects, not just the ones at the player's position
            List<WorldObjectCore> worldObjects =
                this?.core?.worldObjects.Values.SelectMany(worldObject => worldObject.Values)
                    .Where(worldObject => worldObject != null)
                    .Where(worldObject =>
                        !new List<string>
                        {
                            FactoryGameContent.Resources.Iron.ToString(),
                            FactoryGameContent.Resources.Copper.ToString(),
                            FactoryGameContent.Resources.Coal.ToString(),
                        }.Contains(worldObject.worldObjectType)
                    )
                    .ToList() ?? new List<WorldObjectCore>();

            // Display the status data in the UI
            this.StatusUIRightComponent.Display(worldObjects);
        }
    }
}
