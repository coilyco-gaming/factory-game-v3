using System;
using System.Collections;
using System.Collections.Generic;
using System.Linq;
using Assets.Scripts.Components.Unity;
using Assets.Scripts.Core;
using Assets.Scripts.UI;
using Assets.Scripts.Unity;
using TMPro;
using UnityEngine;
using UnityEngine.UI;

namespace Assets.Scripts.FactoryGame
{
    public class FactoryGameController : GameController
    {
        public GameObject resetButton;
        public GameObject pauseButton;
        public GameObject StatusUILeft;
        public GameObject StatusUIRight;
        public float statusUpdateInterval = 0.1f;
        public uint HQOreBuffer = 5;
        public uint spawnAttempts = 5;
        public float oreSpawnFactor = 0.5f;
        public uint OreQuantityBase = 2000;
        public uint OreQuantityRange = 1000;
        public override List<string> ExcludeWorldObjectTypeFromStatus =>
            new()
            {
                FactoryGameContent.Resources.IronOre.ToString(),
                FactoryGameContent.Resources.CopperOre.ToString(),
                FactoryGameContent.Resources.Coal.ToString(),
            };

        private TextMeshProUGUI pauseTextComponent;
        private StatusUILeft statusUILeftComponent;
        private StatusUIRight statusUIRightComponent;

        public override void Start()
        {
            base.Start();

            // TODO: validate that ever object has enough space for every item stack
            this.core.gameContent = new FactoryGameContent();

            this.Map = this.GetComponent<SpriteMapComponent>();
            this.Map.Instantiate(this.GetComponent<Canvas>());

            this.PlayerComponent = this.GetComponent<PlayerComponent>();
            this.PlayerComponent.Instantiate((int)this.Map.MapSize.X, (int)this.Map.MapSize.Y);

            Button resetComponent = this.resetButton.GetComponent<Button>();
            resetComponent.onClick.AddListener(this.Reset);

            Button pauseComponent = this.pauseButton.GetComponent<Button>();
            pauseComponent.onClick.AddListener(this.TogglePausePlay);

            this.pauseTextComponent = this.pauseButton.GetComponentInChildren<TextMeshProUGUI>();
            this.RenderPausePlay(true); // start paused

            this.statusUILeftComponent = this.StatusUILeft.GetComponent<StatusUILeft>();
            this.statusUILeftComponent.Instantiate();
            this.StartCoroutine(this.WriteStatusUILeft());

            this.statusUIRightComponent = this.StatusUIRight.GetComponent<StatusUIRight>();
            this.statusUIRightComponent.Instantiate();
            this.StartCoroutine(this.WriteStatusUIRight());
        }

        protected override void Reset()
        {
            base.Reset();
            this.PlayerComponent.Reset();
            this.SpawnOres(FactoryGameContent.Resources.IronOre.ToString());
            this.SpawnOres(FactoryGameContent.Resources.CopperOre.ToString());
            this.SpawnOres(FactoryGameContent.Resources.Coal.ToString());

            // Spawn some initial buildings

            foreach (
                SpawnQueueItem spawnQueueItem in new List<SpawnQueueItem>
                {
                    // Radars, left side (x=0)
                    new(
                        type: "Radar",
                        x: 0,
                        y: 0,
                        xyCentered: true,
                        targetType: "IronOre",
                        targetSubType: "MiningDrill"
                    ),
                    new(
                        type: "Radar",
                        x: 0,
                        y: 1,
                        xyCentered: true,
                        targetType: "CopperOre",
                        targetSubType: "MiningDrill"
                    ),
                    new(
                        type: "Radar",
                        x: 0,
                        y: 2,
                        xyCentered: true,
                        targetType: "Coal",
                        targetSubType: "MiningDrill"
                    ),
                    new(
                        type: "Radar",
                        x: 0,
                        y: 3,
                        xyCentered: true,
                        targetType: "Coal",
                        targetSubType: "CoalPlant"
                    ),
                    // Far Left Trucks (x=-2)
                    new(
                        type: "Truck",
                        x: -1,
                        y: 0,
                        xyCentered: true,
                        targetType: "Deliver",
                        targetSubType: "IronOre"
                    ),
                    new(
                        type: "Truck",
                        x: -1,
                        y: 1,
                        xyCentered: true,
                        targetType: "Deliver",
                        targetSubType: "IronOre"
                    ),
                    new(
                        type: "Truck",
                        x: -1,
                        y: 2,
                        xyCentered: true,
                        targetType: "Deliver",
                        targetSubType: "CopperOre"
                    ),
                    // Democrat trucks (x=-1)
                    new(
                        type: "Truck",
                        x: 1,
                        y: 0,
                        xyCentered: true,
                        targetType: "Deliver",
                        targetSubType: "IronOre"
                    ),
                    new(
                        type: "Truck",
                        x: 1,
                        y: 1,
                        xyCentered: true,
                        targetType: "Deliver",
                        targetSubType: "IronOre"
                    ),
                    new(
                        type: "Truck",
                        x: 1,
                        y: 2,
                        xyCentered: true,
                        targetType: "Deliver",
                        targetSubType: "CopperOre"
                    ),
                    new(
                        type: "Truck",
                        x: 1,
                        y: 3,
                        xyCentered: true,
                        targetType: "Deliver",
                        targetSubType: "Coal"
                    ),
                    // Coal trucks (y=3)
                    new(
                        type: "Truck",
                        x: 2,
                        y: 3,
                        xyCentered: true,
                        targetType: "Deliver",
                        targetSubType: "Coal"
                    ),
                    new(
                        type: "Truck",
                        x: 3,
                        y: 3,
                        xyCentered: true,
                        targetType: "Deliver",
                        targetSubType: "Coal"
                    ),
                    new(
                        type: "Truck",
                        x: 4,
                        y: 3,
                        xyCentered: true,
                        targetType: "Deliver",
                        targetSubType: "Coal"
                    ),
                    // Coal plant
                    new(
                        type: "CoalPlant",
                        targetType: "Coal",
                        x: 2,
                        y: 2,
                        xyCentered: true,
                        resources: new Dictionary<string, uint> { { "Coal", 4000 } }
                    ),
                    // Stone Mining Drill & Foundries (y=0)
                    new(
                        type: "Foundry",
                        x: 2,
                        y: 0,
                        xyCentered: true,
                        targetType: "IronBars",
                        targetSubType: "IronOre"
                    ),
                    new(
                        type: "Foundry",
                        x: 3,
                        y: 0,
                        xyCentered: true,
                        targetType: "CopperBars",
                        targetSubType: "CopperOre"
                    ),
                    new(
                        type: "Foundry",
                        x: 4,
                        y: 0,
                        xyCentered: true,
                        targetType: "IronBars",
                        targetSubType: "IronOre"
                    ),
                    // Factory layer 1 (y=1), left to right product order:
                    // building materials, motors, circuits, frames
                    new(type: "Factory", x: 2, y: 1, xyCentered: true, targetType: "Motors"),
                    new(type: "Factory", x: 4, y: 1, xyCentered: true, targetType: "Frames"),
                    // Factory layer 2 (y=2), left to right product order:
                    // coal plant, factory, mining drills, power lines
                    new(
                        type: "Factory",
                        x: 3,
                        y: 2,
                        xyCentered: true,
                        targetType: "MiningDrill",
                        resources: new Dictionary<string, uint> { { "MiningDrill", 10 } }
                    ),
                    // Coal Plant Factory Chain
                    new(type: "Factory", x: 5, y: 2, xyCentered: true, targetType: "CoalPlant"),
                    new(type: "Factory", x: 5, y: 1, xyCentered: true, targetType: "Circuits"),
                    new(type: "Factory", x: 6, y: 2, xyCentered: true, targetType: "Frames"),
                    new(type: "Factory", x: 6, y: 3, xyCentered: true, targetType: "Motors"),
                    new(
                        type: "Factory",
                        x: 6,
                        y: 1,
                        xyCentered: true,
                        targetType: "BuildingMaterials"
                    ),
                    new(
                        type: "Foundry",
                        x: 7,
                        y: 2,
                        xyCentered: true,
                        targetType: "CopperBars",
                        targetSubType: "CopperOre"
                    ),
                    new(
                        type: "Foundry",
                        x: 7,
                        y: 1,
                        xyCentered: true,
                        targetType: "IronBars",
                        targetSubType: "IronOre"
                    ),
                    new(
                        type: "Foundry",
                        x: 7,
                        y: 3,
                        xyCentered: true,
                        targetType: "IronBars",
                        targetSubType: "IronOre"
                    ),
                    new(
                        type: "Foundry",
                        x: 5,
                        y: 0,
                        xyCentered: true,
                        targetType: "CopperBars",
                        targetSubType: "CopperOre"
                    ),
                    new(type: "MiningDrill", x: 6, y: 0, xyCentered: true, targetType: "Stone"),
                    // Rightmost trucks (x=7)
                    new(
                        type: "Truck",
                        x: 7,
                        y: 0,
                        xyCentered: true,
                        targetType: "Deploy",
                        targetSubType: "CoalPlant"
                    ),
                    new(
                        type: "Truck",
                        x: 7,
                        y: 1,
                        xyCentered: true,
                        targetType: "Deploy",
                        targetSubType: "MiningDrill"
                    ),
                    new(
                        type: "Truck",
                        x: 7,
                        y: 2,
                        xyCentered: true,
                        targetType: "Deploy",
                        targetSubType: "MiningDrill"
                    ),
                    new(
                        type: "Truck",
                        x: 7,
                        y: 3,
                        xyCentered: true,
                        targetType: "Deploy",
                        targetSubType: "MiningDrill"
                    ),
                    new(
                        type: "Truck",
                        x: 7,
                        y: 4,
                        xyCentered: true,
                        targetType: "Deploy",
                        targetSubType: "MiningDrill"
                    ),
                }
            )
            {
                this.Spawn(spawnQueueItem);
            }

            this.readyForTicks = false;
        }

        private void SpawnOres(string objectName)
        {
            int oresToSpawn = (int)(
                Mathf.Sqrt(
                    (float)(Math.Pow(this.Map.MapSize.X, 2) + Math.Pow(this.Map.MapSize.Y, 2))
                ) * this.oreSpawnFactor
            );
            for (int i = 0; i < oresToSpawn; i++)
            {
                for (int attempts = 0; attempts < this.spawnAttempts; attempts++)
                {
                    // Start by picking a random position
                    int x = this.random.Next(0, (int)this.Map.MapSize.X);
                    int y = this.random.Next(0, (int)this.Map.MapSize.Y);

                    // Draw a triangle to ensure that our ore is at least 2 blocks away
                    uint xOffset = (uint)Math.Abs(x - (this.Map.MapSize.X / 2));
                    uint yOffset = (uint)Math.Abs(y - (this.Map.MapSize.Y / 2));
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
                                FactoryGameContent.Resources.IronOre.ToString(),
                                FactoryGameContent.Resources.CopperOre.ToString(),
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
            // this.PlayerComponent.ToggleFogPosition(paused); // if paused, then move fog closer
        }

        private IEnumerator WriteStatusUILeft()
        {
            while (true)
            {
                yield return new WaitForSeconds(this.statusUpdateInterval);
                try
                {
                    // Get the list of objects at the player's position
                    IEnumerable<WorldObjectCore> worldObjects = this
                        ?.core?.worldObjects?.GetValueOrDefault(
                            this.PlayerComponent.GetGridPosition(),
                            null
                        )
                        ?.Values.ToList();

                    // Display the status data in the UI
                    this.statusUILeftComponent.Display(
                        worldObjects,
                        this.PlayerComponent.GetGridPosition()
                    );
                }
                catch (Exception exc)
                {
                    Debug.Log("Error writing status UI left");
                    Debug.Log(exc);
                }
            }
        }

        private IEnumerator WriteStatusUIRight()
        {
            while (true)
            {
                yield return new WaitForSeconds(this.statusUpdateInterval);
                try
                {
                    // Get the list of all objects
                    IEnumerable<WorldObjectCore> worldObjects = this
                        ?.core?.worldObjects.Values.SelectMany(worldObject => worldObject.Values)
                        .Where(worldObject => worldObject != null)
                        .Where(worldObject =>
                            !this.ExcludeWorldObjectTypeFromStatus.Contains(
                                worldObject.worldObjectType
                            )
                        );

                    // Display the status data in the UI
                    this.statusUIRightComponent.Display(this, worldObjects);
                }
                catch (Exception exc)
                {
                    Debug.Log("Error writing status UI right");
                    Debug.Log(exc);
                }
            }
        }
    }
}
