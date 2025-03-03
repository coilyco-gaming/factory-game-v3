using System;
using System.Collections.Generic;
using System.Linq;
using Assets.Scripts.Components.Unity;
using Assets.Scripts.Core;
using Assets.Scripts.WorldObjects.Core;
using Assets.Scripts.WorldObjects.FactoryGame;
using TMPro;
using UnityEngine;
using UnityEngine.UI;

namespace Assets.Scripts.Unity
{
    public class FactoryGameController : GameController
    {
        public GameObject resetButton;
        public GameObject pauseButton;
        public int HQOreBuffer = 5; // TODO: world init settings screen for these values
        public int spawnAttempts = 5;
        public float oreSpawnFactor = 0.5f;
        public int OreQuantityBase = 2000;
        public int OreQuantityRange = 1000;
        private TextMeshProUGUI pauseTextComponent;

        public override void Start()
        {
            base.Start();

            this.Map = this.GetComponent<SpriteMapComponent>();
            this.Map.Instantiate(this.GetComponent<Canvas>());

            this.PlayerComponent = this.GetComponent<PlayerComponent>();
            this.PlayerComponent.Instantiate(this.Map.mapSize.x, this.Map.mapSize.y);

            this.StatusUIComponent = this.GetComponent<StatusUIComponent>();
            this.StatusUIComponent.Instantiate(this.userInterface);

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
            this.WriteStatusUI();
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
            //
            //   W F F F
            // R C R X X
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

            System.Numerics.Vector2 HQPosition = new(
                this.Map.mapSize.x / 2,
                this.Map.mapSize.y / 2
            );

            this.Spawn(
                new GameControllerCore.SpawnQueueItem(
                    FactoryGameContent.Spawnables.Radar.ToString(),
                    new System.Numerics.Vector2(HQPosition.X, HQPosition.Y),
                    postInstantiateCallback: this.SpawnRadarCallback(
                        FactoryGameContent.Resources.Coal.ToString()
                    )
                )
            );

            this.Spawn(
                new GameControllerCore.SpawnQueueItem(
                    FactoryGameContent.Spawnables.Radar.ToString(),
                    new System.Numerics.Vector2(HQPosition.X, HQPosition.Y + 1),
                    postInstantiateCallback: this.SpawnRadarCallback(
                        FactoryGameContent.Resources.Copper.ToString()
                    )
                )
            );

            this.Spawn(
                new GameControllerCore.SpawnQueueItem(
                    FactoryGameContent.Spawnables.Radar.ToString(),
                    new System.Numerics.Vector2(HQPosition.X + 2, HQPosition.Y + 1),
                    postInstantiateCallback: this.SpawnRadarCallback(
                        FactoryGameContent.Resources.Iron.ToString()
                    )
                )
            );

            this.Spawn(
                new GameControllerCore.SpawnQueueItem(
                    FactoryGameContent.Spawnables.CoalPlant.ToString(),
                    new System.Numerics.Vector2(HQPosition.X + 1, HQPosition.Y),
                    postInstantiateCallback: this.SpawnCoalPlantCallback()
                )
            );

            this.Spawn(
                new GameControllerCore.SpawnQueueItem(
                    FactoryGameContent.Spawnables.CoalPlant.ToString(),
                    new System.Numerics.Vector2(HQPosition.X + 1, HQPosition.Y + 1)
                )
            );

            this.Spawn(
                new GameControllerCore.SpawnQueueItem(
                    FactoryGameContent.Spawnables.Factory.ToString(),
                    new System.Numerics.Vector2(HQPosition.X + 2, HQPosition.Y),
                    instantiateCallback: this.SpawnFactoryCallback(
                        FactoryGameContent.Products.Frames.ToString()
                    )
                )
            );

            this.Spawn(
                new GameControllerCore.SpawnQueueItem(
                    FactoryGameContent.Spawnables.Factory.ToString(),
                    new System.Numerics.Vector2(HQPosition.X + 3, HQPosition.Y),
                    instantiateCallback: this.SpawnFactoryCallback(
                        FactoryGameContent.Products.Circuits.ToString()
                    )
                )
            );

            this.Spawn(
                new GameControllerCore.SpawnQueueItem(
                    FactoryGameContent.Spawnables.Factory.ToString(),
                    new System.Numerics.Vector2(HQPosition.X + 4, HQPosition.Y),
                    instantiateCallback: this.SpawnFactoryCallback(
                        FactoryGameContent.Products.Motors.ToString()
                    )
                )
            );

            this.Spawn(
                new GameControllerCore.SpawnQueueItem(
                    FactoryGameContent.Spawnables.Factory.ToString(),
                    new System.Numerics.Vector2(HQPosition.X + 5, HQPosition.Y),
                    instantiateCallback: this.SpawnFactoryCallback(
                        FactoryGameContent.Products.BuildingMaterials.ToString()
                    )
                )
            );

            this.Spawn(
                new GameControllerCore.SpawnQueueItem(
                    FactoryGameContent.Spawnables.StorageWarehouse.ToString(),
                    new System.Numerics.Vector2(HQPosition.X + 1, HQPosition.Y - 1),
                    postInstantiateCallback: this.SpawnCoalWarehouseCallback()
                )
            );

            this.Spawn(
                new GameControllerCore.SpawnQueueItem(
                    FactoryGameContent.Spawnables.StorageWarehouse.ToString(),
                    new System.Numerics.Vector2(HQPosition.X + 1, HQPosition.Y + 2),
                    postInstantiateCallback: this.SpawnCoalWarehouseCallback()
                )
            );

            this.Spawn(
                new GameControllerCore.SpawnQueueItem(
                    FactoryGameContent.Spawnables.StorageWarehouse.ToString(),
                    new System.Numerics.Vector2(HQPosition.X + 2, HQPosition.Y - 1),
                    postInstantiateCallback: this.SpawnFactoryWarehouseCallback()
                )
            );

            this.Spawn(
                new GameControllerCore.SpawnQueueItem(
                    FactoryGameContent.Spawnables.StorageWarehouse.ToString(),
                    new System.Numerics.Vector2(HQPosition.X + 3, HQPosition.Y - 1),
                    postInstantiateCallback: this.SpawnFactoryWarehouseCallback()
                )
            );

            this.Spawn(
                new GameControllerCore.SpawnQueueItem(
                    FactoryGameContent.Spawnables.StorageWarehouse.ToString(),
                    new System.Numerics.Vector2(HQPosition.X + 4, HQPosition.Y - 1),
                    postInstantiateCallback: this.SpawnFactoryWarehouseCallback()
                )
            );

            this.Spawn(
                new GameControllerCore.SpawnQueueItem(
                    FactoryGameContent.Spawnables.StorageWarehouse.ToString(),
                    new System.Numerics.Vector2(HQPosition.X + 5, HQPosition.Y - 1),
                    postInstantiateCallback: this.SpawnFactoryWarehouseCallback()
                )
            );

            this.Spawn(
                new GameControllerCore.SpawnQueueItem(
                    FactoryGameContent.Spawnables.TransferWarehouse.ToString(),
                    new System.Numerics.Vector2(HQPosition.X + 3, HQPosition.Y + 1)
                )
            );

            this.Spawn(
                new GameControllerCore.SpawnQueueItem(
                    FactoryGameContent.Spawnables.TransferWarehouse.ToString(),
                    new System.Numerics.Vector2(HQPosition.X + 4, HQPosition.Y + 1)
                )
            );

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
                    double distance = Math.Sqrt(Math.Pow(x, 2) + Math.Pow(y, 2));
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

                    // Spawn Ore
                    if (!oreAtPosition)
                    {
                        this.Spawn(
                            new GameControllerCore.SpawnQueueItem(
                                objectName,
                                gridPosition,
                                instantiateCallback: this.SpawnOreCallback()
                            )
                        );
                        break;
                    }
                }
            }
        }

        private Action<GameControllerCore, WorldObjectCore> SpawnFactoryCallback(string product)
        {
            return (gameController, worldObject) =>
            {
                WorldObjectFactory worldObjectFactory = worldObject.backref as WorldObjectFactory;
                worldObjectFactory.productType = product;
            };
        }

        private Action<GameControllerCore, WorldObjectCore> SpawnCoalPlantCallback()
        {
            return (gameController, worldObject) =>
            {
                WorldObjectCoalPlant worldObjectCoalPlant =
                    worldObject.backref as WorldObjectCoalPlant;
                worldObjectCoalPlant.core.Resources.CreateResources(
                    FactoryGameContent.Resources.Coal.ToString(),
                    100 // jumpstart the inserters
                );
            };
        }

        private Action<GameControllerCore, WorldObjectCore> SpawnRadarCallback(string target)
        {
            return (gameController, worldObject) =>
            {
                WorldObjectRadar worldObjectRadar = worldObject.backref as WorldObjectRadar;
                worldObjectRadar.Target = target;
            };
        }

        private Action<GameControllerCore, WorldObjectCore> SpawnOreCallback()
        {
            return (gameController, worldObject) =>
            {
                float randomPercent = (float)this.random.Next(-100, 100) / 100;
                int oreQuantityChange = (int)(randomPercent * this.OreQuantityRange);
                uint oreQuantity = (uint)(this.OreQuantityBase + oreQuantityChange);
                WorldObjectOre worldObjectOre = worldObject.backref as WorldObjectOre;
                worldObjectOre.Amount = oreQuantity;
            };
        }

        private Action<GameControllerCore, WorldObjectCore> SpawnCoalWarehouseCallback()
        {
            return (gameController, worldObject) =>
            {
                WorldObjectStorageWarehouse worldObjectWarehouse =
                    worldObject.backref as WorldObjectStorageWarehouse;
                worldObjectWarehouse.core.Resources.CreateResources(
                    FactoryGameContent.Resources.Coal.ToString(),
                    5000
                );
            };
        }

        private Action<GameControllerCore, WorldObjectCore> SpawnFactoryWarehouseCallback()
        {
            return (gameController, worldObject) =>
            {
                WorldObjectStorageWarehouse worldObjectWarehouse =
                    worldObject.backref as WorldObjectStorageWarehouse;
                worldObjectWarehouse.core.Resources.CreateResources(
                    FactoryGameContent.Resources.Iron.ToString(),
                    2000
                );
                worldObjectWarehouse.core.Resources.CreateResources(
                    FactoryGameContent.Resources.Stone.ToString(),
                    1000
                );
                worldObjectWarehouse.core.Resources.CreateResources(
                    FactoryGameContent.Resources.Copper.ToString(),
                    500
                );
            };
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

        private void WriteStatusUI()
        {
            // Update the UI state with whatever the player is looking at

            // Get the list of objects at the player's position
            System.Numerics.Vector2 position = this.PlayerComponent.GetGridPosition();

            // TODO: grab the nearby objects, not just the ones at the player's position
            List<WorldObjectCore> worldObjects =
                this.core.worldObjects.GetValueOrDefault(position, null)?.Values.ToList()
                ?? new List<WorldObjectCore>();

            // Display the status data in the UI
            this.StatusUIComponent.Display(worldObjects);
        }
    }
}
