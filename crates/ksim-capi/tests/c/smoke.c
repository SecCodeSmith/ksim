#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include "ksim.h"

int main(int argc, char *argv[]) {
    if (argc != 2) {
        fprintf(stderr, "Usage: %s <scenario_path>\n", argv[0]);
        return 1;
    }

    const char *scenario_path = argv[1];
    const uint32_t k = 3;

    printf("ksim version: %s\n", ksim_version());

    struct KsimEnv *env = ksim_env_new(scenario_path, k);
    if (env == NULL) {
        const char *error = ksim_last_error();
        if (error != NULL) {
            fprintf(stderr, "Failed to create environment: %s\n", error);
        } else {
            fprintf(stderr, "Failed to create environment: unknown error\n");
        }
        return 1;
    }

    size_t action_len = ksim_env_action_len(env);
    if (action_len == 0) {
        fprintf(stderr, "Invalid action_len: 0\n");
        ksim_env_free(env);
        return 1;
    }

    size_t observation_len = ksim_env_observation_len(env);
    if (observation_len == 0) {
        fprintf(stderr, "Invalid observation_len: 0\n");
        ksim_env_free(env);
        return 1;
    }

    float *observation = (float *)malloc(observation_len * sizeof(float));
    if (observation == NULL) {
        fprintf(stderr, "Failed to allocate observation buffer\n");
        ksim_env_free(env);
        return 1;
    }

    uint32_t *action = (uint32_t *)malloc(action_len * sizeof(uint32_t));
    if (action == NULL) {
        fprintf(stderr, "Failed to allocate action buffer\n");
        free(observation);
        ksim_env_free(env);
        return 1;
    }

    int reset_result = ksim_env_reset(env, 1, observation, observation_len);
    if (reset_result != KSIM_OK) {
        fprintf(stderr, "Reset failed: %d\n", reset_result);
        free(action);
        free(observation);
        ksim_env_free(env);
        return 1;
    }

    for (size_t i = 0; i < action_len; i++) {
        action[i] = k;
    }

    uint64_t step_count = 0;
    double total_reward = 0.0;
    int step_result;
    double reward;
    uint8_t terminated;
    uint8_t truncated;

    while (1) {
        step_result = ksim_env_step(
            env,
            action,
            action_len,
            observation,
            observation_len,
            &reward,
            &terminated,
            &truncated
        );

        if (step_result != KSIM_OK) {
            const char *error = ksim_last_error();
            if (error != NULL) {
                fprintf(stderr, "Step failed: %s\n", error);
            } else {
                fprintf(stderr, "Step failed with code: %d\n", step_result);
            }
            free(action);
            free(observation);
            ksim_env_free(env);
            return 1;
        }

        step_count++;
        total_reward += reward;

        if (truncated) {
            break;
        }
    }

    printf("Step count: %lu\n", step_count);
    printf("Total reward: %f\n", total_reward);

    free(action);
    free(observation);
    ksim_env_free(env);

    if (step_count == 0) {
        fprintf(stderr, "No steps executed\n");
        return 1;
    }

    return 0;
}
